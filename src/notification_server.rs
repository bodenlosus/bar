use glib::{self};
use gtk::gio;
use std::{
    cell::{RefCell},
    rc::Rc,
};
type NotificationCallback = dyn Fn(&Notification) + 'static;
type NotificationClosedCallback = dyn Fn(u32) + 'static;
type NotificationReplacedCallback = dyn Fn(u32, &Notification) + 'static;

const NOTIFICATION_DBUS_NAME: &str = "org.freedesktop.Notifications";
const NOTIFICATION_DBUS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATION_DBUS_INTERFACE: &str = "org.freedesktop.Notifications";
const NOTIFICATION_INTROSPECTION_XML: &str = include_str!("notifications-introspect.xml");

#[derive(Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: glib::VariantDict,
    pub expire_timeout: i32,
}
#[derive(Clone)]
pub struct NotificationServer {
    on_notification: Option<Rc<NotificationCallback>>,
    on_notification_closed: Option<Rc<NotificationClosedCallback>>,
    on_notification_replaced: Option<Rc<NotificationReplacedCallback>>,
    next_id: Rc<RefCell<u32>>,
}

impl NotificationServer {
    pub fn new() -> Self {
        NotificationServer {
            on_notification: None,
            on_notification_closed: None,
            on_notification_replaced: None,
            next_id: Rc::new(RefCell::new(1)),
        }
    }
    pub fn on_notification<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&Notification) + 'static,
    {
        self.on_notification = Some(Rc::new(callback));
        self
    }

    pub fn on_notification_closed<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u32) + 'static,
    {
        self.on_notification_closed = Some(Rc::new(callback));
        self
    }

    pub fn on_notification_replaced<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(u32, &Notification) + 'static,
    {
        self.on_notification_replaced = Some(Rc::new(callback));
        self
    }

    pub fn connect_to_dbus(&self) -> Result<(), glib::Error> {
        let node_info = gio::DBusNodeInfo::for_xml(NOTIFICATION_INTROSPECTION_XML)?;
        let on_notification_inner = self.on_notification.clone();
        let next_id_inner = self.next_id.clone();
        let _ = gio::bus_own_name(
            gio::BusType::Session,
            NOTIFICATION_DBUS_NAME,
            gio::BusNameOwnerFlags::NONE,
            move |bus_connection, _| {

                let interface_info = match node_info.interfaces().first() {
                    Some(info) => info,
                    None => return,
                };

                let value = next_id_inner.clone();
                let on_notification_inner = on_notification_inner.clone();
                let handle_method_call =
                    move |_connection: gio::DBusConnection,
                          _sender: Option<&str>,
                          _object_path: &str,
                          _interface_name: Option<&str>,
                          method_name: &str,
                          parameters: glib::Variant,
                          invocation: gio::DBusMethodInvocation| {
                        match method_name {
                            "GetServerInformation" => {
                                let server_info = glib::Variant::tuple_from_iter([
                                    glib::Variant::from("My Rust Server"), // Server name
                                    glib::Variant::from("Rust GTK"),       // Vendor
                                    glib::Variant::from("1.0"),            // Version
                                    glib::Variant::from("1.2"),            // Spec version
                                ]);
                                invocation.return_value(Some(&server_info));
                            }
                            "Notify" => {

                                let app_name = parameters.child_get::<String>(0);
                                let replaces_id = parameters.child_get::<u32>(1);
                                let app_icon = parameters.child_get::<String>(2);
                                let summary = parameters.child_get::<String>(3);
                                let body = parameters.child_get::<String>(4);
                                let actions = parameters.child_get::<Vec<String>>(5);
                                let hints =
                                    parameters.child_get::<glib::VariantDict>(6);
                                let expire_timeout = parameters.child_get::<i32>(7);

                                let current_id = if replaces_id == 0 {
                                    let id = value.clone();
                                    let mut id = id.borrow_mut();
                                    let current = *id;
                                    *id = current.wrapping_add(1);
                                    current
                                } else {
                                    replaces_id
                                };

                                let notification = Notification {
                                    id: current_id,
                                    app_name,
                                    replaces_id,
                                    app_icon,
                                    summary,
                                    body,
                                    actions,
                                    hints,
                                    expire_timeout,
                                };

                                let invoc_return = glib::Variant::tuple_from_iter(&[
                                    glib::Variant::from(current_id),
                                ]);

                                invocation.return_value(Some(&invoc_return));
    
                                if let Some(ref callback) = on_notification_inner.clone() {
                                    callback(&notification);
                                }
                            }
                            _ => {}
                        }
                    };

                let res = bus_connection
                    .register_object(NOTIFICATION_DBUS_PATH, interface_info)
                    .method_call(handle_method_call)
                    .build();
                match res {
                    Ok(res) => res,
                    Err(err) => {
                        println!("Error building object: {}", err);
                        return;
                    }
                };
            },
            |x, y| {
                println!("Name acquired {x:?} {y}");
            },
            |x, y| {
                println!("Name lost {x:?} {y}");
            },
        );
        Ok(())
    }
}
