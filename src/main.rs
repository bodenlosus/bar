mod bar;
mod events;
mod notification;
mod notification_server;
mod utils;
mod modules;

use gtk::prelude::*;
use async_channel::{self};
use glib::{self};
use gtk::{self, gio::prelude::ApplicationExt};


pub const ID: &str = "io.github.bodenlosus.panel";

pub const NAME: &str = "bar";

fn load_default_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    glib::set_program_name(Some(NAME));
    glib::set_application_name(NAME);

    let app = gtk::Application::new(Some(ID), Default::default());
    app.connect_startup(|_| {
        load_default_css();
    });
    

    app.connect_activate(|app| {
        let (s_ui, r_ui) = async_channel::unbounded::<events::UIEvent>();

        let mut bar = bar::Bar::new(app, s_ui.clone(), r_ui.clone());

        let stack = modules::ModuleStack::new(gtk::Orientation::Horizontal);
        let time_mod = modules::TimeModule::new();
        stack.add_module(time_mod);
        let notification_mod = modules::Notifications::new();
        stack.add_module(notification_mod);

        bar.add_module(stack, bar::Align::Center, false);
        bar.add_module(time_mod, , add_widget);
        bar.event_loop();
        bar.show();

        let not_server = async move {
            let not_server = notification_server::NotificationServer::new(s_ui.clone());
            if let Err(e) = not_server.connect_to_dbus() {
                eprintln!("Error connecting to D-Bus: {e:?}");
            }
        };
        //spawn the notification server in a seperate thread

        glib::MainContext::default().spawn(not_server);

    });
    app.run();
}