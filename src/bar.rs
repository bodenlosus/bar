use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::time;



use crate::events::UIEvent;
use crate::notification::NotificationObject;
use crate::notification_server;
use crate::utils::{self, unwrap_or_return};

use adw::prelude::AdwApplicationWindowExt;
use adw::{self};
use async_channel::{Receiver, Sender};
use cascade::cascade;
use glib::{self};
use gtk::{gio, Popover};
use gtk::{self, prelude::*};
use layer_shell::{self, Edge, Layer, LayerShell};
use crate::modules::Module;

pub enum Align {
    Start = 0,
    Center = 1,
    End = 2,
}

pub struct Bar {
    pub window: gtk::ApplicationWindow,
    pub layout: (gtk::Box, gtk::Box, gtk::Box),
    // pub modules: Modules,
    pub s_ui: Sender<UIEvent>,
    pub r_ui: Receiver<UIEvent>,
    pub modules: RefCell<HashMap<String, Rc<dyn Module>>>,
}
impl Bar {
    pub fn new(app: &gtk::Application, s_ui: Sender<UIEvent>, r_ui: Receiver<UIEvent>) -> Self {
        let center_box = gtk::CenterBox::new();
        // let time_mod = TimeModule::new();

        let start = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let middle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let end = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // middle.append(&time_mod.widget);

        center_box.set_start_widget(Some(&start));
        center_box.set_center_widget(Some(&middle));
        center_box.set_end_widget(Some(&end));

        let window = cascade! {
        gtk::ApplicationWindow::new(app);
            ..init_layer_shell();
            ..set_anchor(Edge::Top, true);
            ..set_anchor(Edge::Right, true);
            ..set_anchor(Edge::Left, true);
            ..auto_exclusive_zone_enable();
            ..set_height_request(30);
            ..set_child(Some(&center_box));
        };

        Self {
            window,
            layout: (start, middle, end),
            s_ui,
            r_ui,
            modules: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_module(&self, name: &str) -> Option<Rc<dyn Module>> {
        self.modules.borrow().get(name).cloned()
    }

    pub fn add_module<T, M>(&self, name: T, module: M, align: Align)
    where 
        T: Into<String>,
        M: Module + 'static,
      {
        let name = name.into();
        let module = Rc::new(module);
        let widget = create_module_container(module.clone());
        match align {
            Align::Start => self.layout.0.append(&widget),
            Align::Center => self.layout.1.append(&widget),
            Align::End => self.layout.2.append(&widget),
        }

        self.modules
            .borrow_mut()
            .insert(name.clone(), module);
        
    }

    pub fn event_loop(&mut self) {
        let s_ui = self.s_ui.clone();
        let r_ui = self.r_ui.clone();

        let time_mod = self.get_module("time");

        glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = r_ui.recv().await {
                match event {
                    UIEvent::Notification(notification) => {
                        if let Some(ref module) = time_mod {
                        }
                    }
                }
            }
        });
    }

    pub fn show(&self) {
        self.window.present()
    }
}

fn create_module_container<M>(module: Rc<M>) -> gtk::MenuButton
where 
    M: Module + 'static,
 {
    
    let label = gtk::Label::new(Some(module.name()));
    let popover = Popover::new();
    
    popover.connect_realize(move |popover| {
        popover.set_has_arrow(false);
        popover.set_offset(0, 10);
        popover.set_child(Some(&module.get_widget()));
    });
    
    let button =cascade! {
        gtk::MenuButton::new();
        ..set_popover(Some(&popover));
        ..set_child(Some(&label));
    };
    button
}

