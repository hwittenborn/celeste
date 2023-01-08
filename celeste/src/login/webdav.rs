//! The data for a WebDAV Rclone config.
use super::{login_util, nextcloud::NextcloudConfig, ServerType};
use crate::mpsc::Sender;
use adw::{
    gtk::{glib, Button},
    prelude::*,
    ApplicationWindow, EntryRow,
};

pub enum WebDavType {
    Nextcloud,
    WebDav,
}

#[derive(Clone, Debug, Default)]
pub struct WebDavConfig {
    pub server_name: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
}

impl super::LoginTrait for WebDavConfig {
    fn get_sections(
        _window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        Self::webdav_sections(sender, WebDavType::WebDav)
    }
}

impl WebDavConfig {
    pub fn webdav_sections(
        sender: Sender<Option<ServerType>>,
        webdav_type: WebDavType,
    ) -> (Vec<EntryRow>, Button) {
        let mut sections: Vec<EntryRow> = vec![];

        let server_name = login_util::server_name_input();
        let server_url = login_util::server_url_input();
        let username = login_util::username_input();
        let password = login_util::password_input();
        let submit_button = login_util::submit_button();

        sections.push(server_name.clone());
        sections.push(server_url.clone());
        sections.push(username.clone());
        sections.push(password.clone().into());

        submit_button.connect_clicked(
            glib::clone!(@weak server_name, @weak server_url, @weak username, @weak password => move |_| {
                let server_type = match webdav_type {
                    WebDavType::Nextcloud => ServerType::Nextcloud(NextcloudConfig {
                        server_name: server_name.text().to_string(),
                        server_url: server_url.text().to_string(),
                        username: username.text().to_string(),
                        password: password.text().to_string(),
                    }),
                    WebDavType::WebDav => ServerType::WebDav(WebDavConfig{
                        server_name: server_name.text().to_string(),
                        server_url: server_url.text().to_string(),
                        username: username.text().to_string(),
                        password: password.text().to_string(),
                    })
                };
                sender.send(Some(server_type));
            }),
        );

        server_name.connect_changed(glib::clone!(@weak server_name, @weak server_url, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &server_url, &username, &password.into()], &submit_button)));
        server_url.connect_changed(glib::clone!(@weak server_name, @weak server_url, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &server_url, &username, &password.into()], &submit_button)));
        username.connect_changed(glib::clone!(@weak server_name, @weak server_url, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &server_url, &username, &password.into()], &submit_button)));
        password.connect_changed(glib::clone!(@weak server_name, @weak server_url, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &server_url, &username, &password.into()], &submit_button)));

        (sections, submit_button)
    }
}
