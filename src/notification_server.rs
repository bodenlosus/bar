use async_channel::Sender;
use glib::variant::ToVariant;
use glib::{self};
use gtk::gio;
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::events::{NotificationEvent, UIEvent};
use crate::utils::unwrap_or_return;
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
    pub hints: glib::Variant,
    pub expire_timeout: i32,
}

impl Debug for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notification")
            .field("id", &self.id)
            .field("app_name", &self.app_name)
            .field("replaces_id", &self.replaces_id)
            .field("app_icon", &self.app_icon)
            .field("summary", &self.summary)
            .field("body", &self.body)
            .field("actions", &self.actions)
            .field("expire_timeout", &self.expire_timeout)
            .finish()
    }
}

#[derive(Clone)]
pub struct NotificationServer {
    next_id: Rc<RefCell<u32>>,
    sender: Sender<UIEvent>,
}

impl NotificationServer {
    pub fn new(sender: Sender<UIEvent>) -> Self {
        NotificationServer {
            next_id: Rc::new(RefCell::new(1)),
            sender,
        }
    }

    pub fn connect_to_dbus(&self) -> Result<(), glib::Error> {
        let next_id_inner = self.next_id.clone();
        let sender = self.sender.clone();

        let _ = gio::bus_own_name(
            gio::BusType::Session,
            NOTIFICATION_DBUS_NAME,
            gio::BusNameOwnerFlags::NONE,
            move |bus_connection, _| {
                bus_aquired(bus_connection, next_id_inner.clone(), sender.clone());
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

fn bus_aquired(
    connection: gio::DBusConnection,
    next_id: Rc<RefCell<u32>>,
    sender: Sender<UIEvent>,
) {
    let node_info = unwrap_or_return!(
        gio::DBusNodeInfo::for_xml(NOTIFICATION_INTROSPECTION_XML),
        Result
    );
    let interface_info = unwrap_or_return!(node_info.interfaces().first(), Option);

    let res = connection
        .register_object(NOTIFICATION_DBUS_PATH, interface_info)
        .method_call(
            move |_connection,
                  _sender ,
                  _object_path ,
                  _interface_name,
                  method_name,
                  parameters,
                  invocation| {
                    let next_id = next_id.clone();
                    let sender = sender.clone();
                    let method_name = method_name.to_string();
                    let fut = handle_method_call(
                            // _connection,
                            // _sender,
                            // _object_path,
                            // _interface_name,
                            method_name,
                            parameters,
                            invocation,
                            next_id,
                            sender,
                        );
                    let _ = glib::MainContext::default().spawn_local(fut);
                }
        ,
        )   
        .build();
    match res {
        Ok(res) => res,
        Err(err) => {
            println!("Error building object: {}", err);
            return;
        }
    };
}

async fn handle_method_call(
    // _connection: gio::DBusConnection,
    // _sender: Option<&str>,
    // _object_path: &str,
    // _interface_name: Option<&str>,
    method_name: String,
    parameters: glib::Variant,
    invocation: gio::DBusMethodInvocation,
    next_id: Rc<RefCell<u32>>,
    sender: Sender<UIEvent>,
) {
    match method_name.as_str() {
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


            println!("Unwrap mistt");

            let app_name = parameters.child_get::<String>(0);
            let replaces_id = parameters.child_get::<u32>(1);
            let app_icon = parameters.child_get::<String>(2);
            let summary = parameters.child_get::<String>(3);
            let body = parameters.child_get::<String>(4);
            let actions = parameters.child_get::<Vec<String>>(5);

            let hints = parameters.child_get::<glib::VariantDict>(6);

            let expire_timeout = parameters.child_get::<i32>(7);

            let hints = hints.to_variant();

            let current_id = if replaces_id == 0 {
                let mut id = next_id.borrow_mut();
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

            let invoc_return = glib::Variant::tuple_from_iter(&[glib::Variant::from(current_id)]);

            invocation.return_value(Some(&invoc_return));


            let event = UIEvent::Notification(NotificationEvent::NewNotification(
                notification,
            ));

            if let Err(err) = sender.send(event).await {
                println!("Error sending notification: {}", err);
            }
        }
        _ => {}
    }
}
