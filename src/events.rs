use crate::{notification_server};

#[derive(Debug)]
pub enum UIEvent {
    Notification(NotificationEvent),
}

#[derive(Debug)]
pub enum NotificationEvent {
    NewNotification(notification_server::Notification),
}