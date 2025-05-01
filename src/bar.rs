use crate::events::UIEvent;
use crate::utils;

use adw::{self, prelude::*};
use async_channel::{Receiver, Sender};
use cascade::cascade;
use glib::{self, clone};
use gtk::{self, prelude::*};
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

struct TimeModule {
    pub widget: gtk::MenuButton,
    pub label: gtk::Label,
    pub calendar: gtk::Calendar,
    date_time: Option<glib::DateTime>,
}

impl TimeModule {
    pub fn new() -> Self {
        let label = gtk::Label::new(None);
        let date_display = gtk::Label::new(Some("March 18 2025"));
        date_display.set_css_classes(&["date-display"]);
        let calendar = gtk::Calendar::new();

        let popover = cascade! {
            gtk::Popover::new();
            ..set_has_arrow(false);
            ..set_child(Some(&cascade! {
                gtk::Box::new(gtk::Orientation::Vertical, 5);
                ..append(&date_display);
                ..append(&calendar);

            }));
            ..set_offset(0, 10);
        };

        let button = cascade! {
            gtk::MenuButton::new();
            ..set_child(Some(&label));
            ..set_popover(Some(&popover));
        };

        Self {
            widget: button,
            label,
            calendar,
            date_time: glib::DateTime::now_local().ok(),
        }
    }
    pub fn set_datetime(&mut self, datetime: &glib::DateTime) {
        self.date_time = Some(datetime.clone());
        let text = datetime
            .format("%a %e %b %H:%M:%S")
            .map(|s| s.to_string())
            .unwrap_or("".to_string());
        self.label.set_text(&text);
    }
}
