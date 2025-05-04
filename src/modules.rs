use crate::{notification::NotificationObject, utils::unwrap_or_return};
use cascade::cascade;
use glib::object::{Cast, IsA};
use gtk::prelude::*;
use gtk::{self, gio};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

enum ModuleType {
    Time,
    Notifications,
    Stack,
}

pub trait Module {
    fn name(&self) -> &str;
    fn get_widget(&self) -> gtk::Widget;
    fn get_button(&self) -> Option<gtk::MenuButton> {
        None
    }
}

pub struct ModuleStack {
    modules: RefCell<Vec<Rc<dyn Module>>>,
    orientation: gtk::Orientation,
    widget: RefCell<Option<gtk::Box>>,
}

impl ModuleStack {
    pub fn new(orientation: gtk::Orientation) -> Self {
        Self {
            modules: RefCell::new(Vec::new()),
            orientation,
            widget: RefCell::new(None),
        }
    }

    pub fn add_module<M>(&self, module: M)
    where
        M: Module + 'static,
    {
        let module = Rc::new(module);
        self.modules.borrow_mut().push(module);
    }
}

impl Module for ModuleStack {
    fn name(&self) -> &str {
        "ModuleStack"
    }

    fn get_widget(&self) -> gtk::Widget {
        let mut widget = self.widget.borrow_mut();
        if let Some(widget) = widget.as_ref() {
            return widget.clone().upcast::<gtk::Widget>();
        }

        let container = gtk::Box::new(self.orientation, 10);
        for module in self.modules.borrow().iter() {
            let child = module.get_widget();
            container.append(&child);
        }
        *widget = Some(container.clone());
        container.upcast::<gtk::Widget>()
    }
    fn get_button(&self) -> Option<gtk::MenuButton> {
        let button = gtk::MenuButton::new();
        let label = gtk::Label::new(Some("Modules"));
        button.set_child(Some(&label));
        Some(button)
    }
}

pub struct TimeModule {
    widget: RefCell<Option<gtk::Box>>,
}

impl TimeModule {
    pub fn new() -> Self {
        Self {
            widget: RefCell::new(None),
        }
    }
    // pub fn set_datetime(&mut self, datetime: &glib::DateTime) {
    //     self.date_time = Some(datetime.clone());

    //     let text = datetime
    //         .format("%c")
    //         .map(|s| s.to_string())
    //         .unwrap_or("".to_string());
    //     self.label.set_markup(&text);

    //     let date_text = datetime.format("%x").unwrap_or_default();
    //     let weekday_text = datetime.format("%A").unwrap_or_default();
    //     self.date_display.set_markup(&format!(
    //         "<span size='large'>{}</span>\r<span size='x-large'>{}</span>",
    //         weekday_text, date_text
    //     ));
    // }
}

impl Module for TimeModule {
    fn name(&self) -> &str {
        "TimeModule"
    }

    fn get_widget(&self) -> gtk::Widget {
        let mut widget = self.widget.borrow_mut();

        if let Some(widget) = widget.as_ref() {
            return widget.clone().upcast::<gtk::Widget>();
        }

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

        let container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 5);
            ..append(&date_container);
            ..append(&calendar);
        };
        *widget = Some(container.clone());

        container.upcast::<gtk::Widget>()
    }
}

#[derive(Clone)]
pub struct Notifications {
    widget: RefCell<Option<gtk::ListView>>,
    store: gio::ListStore,
}

impl Notifications {
    pub fn new() -> Self {
        let store = gio::ListStore::new::<NotificationObject>();

        Self {
            widget: RefCell::new(None),
            store,
        }
    }
}

impl Module for Notifications {
    fn name(&self) -> &str {
        "Notifications"
    }

    fn get_widget(&self) -> gtk::Widget {
        let mut widget = self.widget.borrow_mut();

        if let Some(widget) = widget.as_ref() {
            return widget.clone().upcast::<gtk::Widget>();
        }

        let selection_model = gtk::NoSelection::new(Some(self.store.clone()));
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            if let Some(item) = item.downcast_ref::<gtk::ListItem>() {
                item.set_child(Some(&create_notification_widget()));
            };
        });

        factory.connect_bind(move |_, item| {
            let (child, notif) = unwrap_or_return!(
                downcast_list_item::<gtk::Box, NotificationObject>(item),
                Option
            );

            let header =
                unwrap_or_return!(child.first_child().and_downcast::<gtk::Label>(), Option);
            let body =
                unwrap_or_return!(header.next_sibling().and_downcast::<gtk::Label>(), Option);
            let time = unwrap_or_return!(body.next_sibling().and_downcast::<gtk::Label>(), Option);

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

        *widget = Some(container.clone());

        container.upcast::<gtk::Widget>()
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
