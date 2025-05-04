use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::time;



use crate::events::{NotificationEvent, UIEvent};
use crate::notification::NotificationObject;
use crate::{modules, notification_server};
use crate::utils::{self, unwrap_or_return};

use adw::prelude::AdwApplicationWindowExt;
use adw::{self};
use async_channel::{Receiver, Sender};
use cascade::cascade;
use glib::{self};
use gtk::{gio, Popover};
use gtk::{self, prelude::*};
use layer_shell::{self, Edge, Layer, LayerShell};
use crate::modules::{Module, ModuleType};

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
    pub modules: RefCell<HashMap<ModuleType, Rc<dyn Module>>>,
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

    pub fn add_module<M>(&self, module: M, align: Align, add_widget: bool)
    where 
        M: Module + 'static,
      {
        let module = Rc::new(module);
        let widget = create_module_container(module.clone());

        self.modules
            .borrow_mut()
            .insert(module.get_type(), module);
        
        if !add_widget {
            return;
        }
        match align {
            Align::Start => self.layout.0.append(&widget),
            Align::Center => self.layout.1.append(&widget),
            Align::End => self.layout.2.append(&widget),
        }    
    }
    pub fn get_module(&self, module_type: ModuleType) -> Option<Rc<dyn Module>> {
        self.modules.borrow().get(&module_type).cloned()
    }
    pub fn event_loop(&mut self) {
        let s_ui = self.s_ui.clone();
        let r_ui = self.r_ui.clone();

        let not_mod = self.get_module(ModuleType::Notifications);

        glib::MainContext::default().spawn_local(async move {
            let not_mod = not_mod.clone();
            while let Ok(event) = r_ui.recv().await {
                match event {
                    UIEvent::Notification(event) => {
                        handle_not_event(event, not_mod.clone());
                    }
                }
            };
        });
    }

    pub fn show(&self) {
        self.window.present()
    }
}

fn handle_not_event(
    event: NotificationEvent,
    module: Option<Rc<dyn Module>>,
) {
    let module = unwrap_or_return!(module, Option);
    let module = unwrap_or_return!(
        module.as_any().downcast_ref::<crate::modules::Notifications>(), Option
    );

    match event {
        NotificationEvent::NewNotification(notification) => {
            println!("New notification: {:?}", notification);
            module.add_notification(notification);
        }
        _ => {}
    };
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

