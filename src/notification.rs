

use crate::notification_server::Notification;
mod imp {
    use std::cell::{Cell, RefCell};

    use glib;
    use glib::subclass::types::ObjectSubclass;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::NotificationObject)]
    pub struct NotificationObject {
        #[property(get, set)]
        pub id: Cell<u32>,
        #[property(get, set)]
        pub app_name: RefCell<String>,
        #[property(get, set)]
        pub replaces_id: RefCell<u32>,
        #[property(get, set)]
        pub app_icon: RefCell<String>,
        #[property(get, set)]
        pub summary: RefCell<String>,
        #[property(get, set)]
        pub body: RefCell<String>,
        #[property(get, set)]
        pub actions: RefCell<Vec<String>>,
        #[property(get, set)]
        pub hints: RefCell<glib::VariantDict>,
        #[property(get, set)]
        pub expire_timeout: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationObject {
        const NAME: &'static str = "Notification";
        type Type = super::NotificationObject;
        type ParentType = glib::Object;
        type Interfaces = ();
    }
    #[glib::derived_properties]
    impl ObjectImpl for NotificationObject {}
}

glib::wrapper! {
    pub struct NotificationObject(ObjectSubclass<imp::NotificationObject>);
}

impl NotificationObject {
    pub fn new() -> Self {
        glib::Object::new()
    }
    pub fn set(
        &self, n:Notification, 
    ) {
        self.set_id(n.id);
        self.set_app_name(n.app_name);
        self.set_replaces_id(n.replaces_id);
        self.set_app_icon(n.app_icon);
        self.set_summary(n.summary);
        self.set_body(n.body);
        self.set_actions(n.actions);
        self.set_hints(glib::VariantDict::new(Some(&n.hints)));
        self.set_expire_timeout(n.expire_timeout);
    }

}
