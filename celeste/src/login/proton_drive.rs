//! The data for a Proton Drive Rclone config.
use super::ServerType;
use crate::{login::login_util, mpsc::Sender};
use adw::{glib, gtk::Button, prelude::*, ApplicationWindow, EntryRow, MessageDialog};

#[derive(Clone, Debug, Default)]
pub struct ProtonDriveConfig {
    pub server_name: String,
    pub username: String,
    pub password: String,
}

impl super::LoginTrait for ProtonDriveConfig {
    fn get_sections(
        window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        let mut sections = vec![];
        let server_name = login_util::server_name_input();
        let username = login_util::username_input();
        let password = login_util::password_input();
        let submit_button = login_util::submit_button();

        sections.push(server_name.clone());
        sections.push(username.clone());
        sections.push(password.clone().into());

        submit_button.connect_clicked(
            glib::clone!(@weak server_name, @weak username, @weak password => move |_| {
                sender.send(Some(ServerType::ProtonDrive(ProtonDriveConfig {
                    server_name: server_name.text().to_string(),
                    username: username.text().to_string(),
                    password: password.text().to_string()
                })));
            }),
        );

        server_name.connect_changed(glib::clone!(@weak server_name, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &username, &password.into()], &submit_button)));
        username.connect_changed(glib::clone!(@weak server_name, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &username, &password.into()], &submit_button)));
        password.connect_changed(glib::clone!(@weak server_name, @weak username, @weak password, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &username, &password.into()], &submit_button)));

        (sections, submit_button)
    }
}
