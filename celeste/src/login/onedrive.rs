//! The data for a OneDrive Rclone config.
use super::ServerType;
use crate::{
    login::gdrive::{AuthType, GDriveConfig},
    mpsc::Sender,
};
use adw::{gtk::Button, ApplicationWindow, EntryRow};

static DEFAULT_CLIENT_ID: &str = "2f1c3d96-c564-477d-b692-a6c6ae780ea9";
static DEFAULT_CLIENT_SECRET: &str = "S2W8Q~FdqU-jj0ywBE5J.ndwrIar0diz96nqqaUd";

#[derive(Clone, Debug, Default)]
pub struct OneDriveConfig {
    pub server_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_json: String,
}

impl super::LoginTrait for OneDriveConfig {
    fn get_sections(
        window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        GDriveConfig::auth_sections(
            window,
            sender,
            AuthType::OneDrive,
            DEFAULT_CLIENT_ID.to_owned(),
            DEFAULT_CLIENT_SECRET.to_owned(),
        )
    }
}
