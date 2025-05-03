use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::events::UIEvent;
use crate::notification::NotificationObject;
use crate::notification_server;

use adw::prelude::AdwApplicationWindowExt;
use adw::{self};
use async_channel::{Receiver, Sender};
use cascade::cascade;
use glib::{self, clone};
use gtk::Widget;
use gtk::gio;
use gtk::subclass::selection_model;
use gtk::{self, prelude::*};
use gtk4_sys::GtkListBox;
use layer_shell::{self, Edge, Layer, LayerShell};

pub struct Modules {
    pub time: Option<TimeModule>,
}

pub struct Bar {
    pub window: adw::ApplicationWindow,
    pub layout: (gtk::Box, gtk::Box, gtk::Box),
    pub modules: Modules,
    pub tx: Sender<UIEvent>,
}
impl Bar {
    pub fn new(app: &adw::Application, tx: Sender<UIEvent>) -> Self {
        let center_box = gtk::CenterBox::new();
        let time_mod = TimeModule::new();

        let start = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let middle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let end = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        middle.append(&time_mod.widget);

        center_box.set_start_widget(Some(&start));
        center_box.set_center_widget(Some(&middle));
        center_box.set_end_widget(Some(&end));

        let window = cascade! {
        adw::ApplicationWindow::new(app);
            ..init_layer_shell();
            ..set_anchor(Edge::Top, true);
            ..set_anchor(Edge::Right, true);
            ..set_anchor(Edge::Left, true);
            ..auto_exclusive_zone_enable();
            ..set_height_request(30);
            ..set_content(Some(&center_box));
        };

        Self {
            modules: Modules {
                time: Some(time_mod),
            },
            window,
            layout: (start, middle, end),
            tx,
        }
    }
    pub fn show(&self) {
        self.window.present()
    }
}

pub struct TimeModule {
    pub widget: gtk::MenuButton,
    label: gtk::Label,
    calendar: gtk::Calendar,
    date_display: gtk::Label,
    date_time: Option<glib::DateTime>,
    notifications: Notifications,
}

impl TimeModule {
    pub fn new() -> Self {
        let label = gtk::Label::new(None);

        let date_display = cascade! {
            gtk::Label::new(Some("March 18 2025"));
        };
        let date_container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 5);
            ..append(&date_display);
            ..set_halign(gtk::Align::Start);
            ..set_css_classes(&["date-display"]);
        };
        date_display.set_css_classes(&["date-display"]);
        let calendar = gtk::Calendar::new();

        let time_container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 5);
            ..append(&date_container);
            ..append(&calendar);
        };

        let mut notifications = Notifications::new();

        let popover = cascade! {
            gtk::Popover::new();
            ..set_has_arrow(false);
            ..set_child(Some(&cascade! {
                gtk::Box::new(gtk::Orientation::Horizontal, 5);
                ..append(&notifications.widget);
                ..append(&gtk::Separator::new(gtk::Orientation::Vertical));
                ..append(&time_container);
            }));
            ..set_offset(0, 10);
        };

        let button = cascade! {
            gtk::MenuButton::new();
            ..set_child(Some(&label));
            ..set_popover(Some(&popover));
        };

        notifications.connect_dbus();

        Self {
            notifications,
            date_display,
            widget: button,
            label,
            calendar,
            date_time: glib::DateTime::now_local().ok(),
        }
    }
    pub fn set_datetime(&mut self, datetime: &glib::DateTime) {
        self.date_time = Some(datetime.clone());

        let text = datetime
            .format("%c")
            .map(|s| s.to_string())
            .unwrap_or("".to_string());
        self.label.set_markup(&text);

        let date_text = datetime.format("%x").unwrap_or_default();
        let weekday_text = datetime.format("%A").unwrap_or_default();
        self.date_display.set_markup(&format!(
            "<span size='large'>{}</span>\r<span size='x-large'>{}</span>",
            weekday_text, date_text
        ));
    }
}

#[derive(Clone)]
pub struct Notifications {
    pub widget: gtk::ListView,
    store: gio::ListStore,
}

impl Notifications {
    pub fn new() -> Self {
        let model = gio::ListStore::new::<NotificationObject>();
        let selection_model = gtk::NoSelection::new(Some(model.clone()));
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            if let Some(item) = item.downcast_ref::<gtk::ListItem>() {
                item.set_child(Some(&create_notification_widget()));
            };
        });

        factory.connect_bind(move |_, item| {
            let Some((child, notif)) = downcast_list_item::<gtk::Box, NotificationObject>(item)
            else {
                return;
            };

            let Some(header) = child.first_child().and_downcast::<gtk::Label>() else {
                return;
            };
            let Some(body) = header.next_sibling().and_downcast::<gtk::Label>() else {
                return;
            };
            let Some(time) = body.next_sibling().and_downcast::<gtk::Label>() else {
                return;
            };

            let name = notif.app_name();
            let body_text = notif.body();

            cascade! {
                header;
                ..set_css_classes(&["header"]);
                ..set_ellipsize(gtk::pango::EllipsizeMode::End);
                ..set_label(&name);
            };
            cascade! {
                body;
                ..set_ellipsize(gtk::pango::EllipsizeMode::End);
                ..set_label(&body_text);
                ..set_css_classes(&["body"]);
            };

            if let Ok(dt) = glib::DateTime::now_local() {
                let time_str = dt.format("%H:%M:%S").unwrap_or_default();
                cascade! {
                    time;
                    ..set_ellipsize(gtk::pango::EllipsizeMode::End);
                    ..set_css_classes(&["time"]);
                    ..set_label(&time_str);
                };
            }
        });

        let container = gtk::ListView::new(Some(selection_model), Some(factory));

        container.set_width_request(200);
        container.set_css_classes(&["notification-list"]);

        Self {
            widget: container,
            store: model,
        }
    }

    pub fn connect_dbus(&mut self) {
        let store = self.store.clone();
        let on_notifcation = move |n: &notification_server::Notification| {
            let dt = glib::DateTime::now_local();
            let time_str = {
                if let Ok(dt) = dt {
                    dt.format("%x %H:%M:%S").unwrap_or_default().to_string()
                } else {
                    "".to_string()
                }
            };

            let notif = NotificationObject::new();
            notif.set(n.clone());
            store.insert(n.id - 1, &notif);
        };
        cascade! {
            notification_server::NotificationServer::new();
            ..on_notification(on_notifcation);
            ..on_notification_closed(|id| {
                println!("Notification closed: {}", id);
            });
            ..on_notification_replaced(|id, n| {

            });
            ..connect_to_dbus().unwrap();
        };
    }
}

fn create_notification_widget() -> gtk::Box {
    let header_label = gtk::Label::new(None);
    let body_label = gtk::Label::new(None);
    let time_label = gtk::Label::new(None);

    cascade! {
        gtk::Box::new(gtk::Orientation::Vertical, 5);
        ..append(&header_label);
        ..append(&body_label);
        ..append(&time_label);
        ..set_css_classes(&["notification"]);
    }
}

fn downcast_list_item<T, U>(item: &glib::Object) -> Option<(T, U)>
where
    T: IsA<gtk::Widget>,
    U: IsA<glib::Object>,
{
    let item = item.downcast_ref::<gtk::ListItem>()?;

    let child = item.child().and_downcast::<T>()?;
    let item = item.item().and_downcast::<U>()?;
    Some((child, item))
}
