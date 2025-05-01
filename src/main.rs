mod bar;
mod events;
mod utils;

use adw::prelude::*;
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

    let app = adw::Application::new(Some(ID), Default::default());
    app.connect_startup(|_| {
        load_default_css();
    });
    app.connect_activate(|app| {
        let (tx, trx) = async_channel::unbounded::<events::UIEvent>();

        let mut bar = bar::Bar::new(app, tx);
        bar.show();
        utils::set_interval(
            move || {
                let dt = match glib::DateTime::now_local() {
                    Ok(dt) => dt,
                    Err(e) => {
                        eprintln!("Error retrieving DateTime: {e:?}");
                        return glib::ControlFlow::Continue;
                    }
                };
                if let Some(module) = bar.modules.time.as_mut() {
                    module.set_datetime(&dt);
                }
                glib::ControlFlow::Continue
            },
            250,
        );
    });
    app.run();
}
