//! The data for an Owncloud Rclone config.
use super::{
    webdav::{WebDavConfig, WebDavType},
    ServerType,
};
use crate::mpsc::Sender;
use adw::{gtk::Button, ApplicationWindow, EntryRow};

#[derive(Clone, Debug, Default)]
pub struct OwncloudConfig {
    pub server_name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
}

impl super::LoginTrait for OwncloudConfig {
    fn get_sections(
        _window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        WebDavConfig::webdav_sections(sender, WebDavType::Owncloud)
    }
}
