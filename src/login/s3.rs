//! The data for a S3 Rclone config.
use super::{login_util, ServerType};
use crate::mpsc::Sender;
use adw::{
    gtk::{glib, Button},
    prelude::*,
    ApplicationWindow, EntryRow,
};

#[derive(Clone, Debug, Default)]
pub struct S3Config {
    pub server_name: String,
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub provider: String,
    pub acl: String,
    pub bucket_acl: String,
}
impl super::LoginTrait for S3Config {
    fn get_sections(
        _window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        let mut sections: Vec<EntryRow> = vec![];

        let server_name = login_util::server_name_input();
        let endpoint = login_util::generic_str_input("Endpoint");
        let region = login_util::generic_str_input("Region");
        let access_key_id = login_util::generic_str_input("Login ID");
        let secret_access_key = login_util::generic_str_input("Secret key");

        let submit_button = login_util::submit_button();

        sections.push(server_name.clone());
        sections.push(endpoint.clone());
        sections.push(region.clone());
        sections.push(access_key_id.clone());
        sections.push(secret_access_key.clone());

        submit_button.connect_clicked(
            glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key => move |_| {

                let server_type = ServerType::S3(S3Config {
                        server_name: server_name.text().to_string(),
                        endpoint: endpoint.text().to_string(),
                        region: region.text().to_string(),
                        access_key_id: access_key_id.text().to_string(),
                        secret_access_key: secret_access_key.text().to_string(),
                        // TODO handle these hardcoded values
                        provider: "Other".to_string(),
                        acl: "private".to_string(),
                        bucket_acl: "private".to_string(),
                    });
                sender.send(Some(server_type));
            }),
        );

        server_name.connect_changed(glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &endpoint, &region, &access_key_id, &secret_access_key], &submit_button)));
        endpoint.connect_changed(glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &endpoint, &region, &access_key_id, &secret_access_key], &submit_button)));
        region.connect_changed(glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &endpoint, &region, &access_key_id, &secret_access_key], &submit_button)));
        access_key_id.connect_changed(glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &endpoint, &region, &access_key_id, &secret_access_key], &submit_button)));
        secret_access_key.connect_changed(glib::clone!(@weak server_name, @weak endpoint, @weak region, @weak access_key_id, @weak secret_access_key, @weak submit_button => move |_| login_util::check_responses(&[&server_name, &endpoint, &region, &access_key_id, &secret_access_key], &submit_button)));

        (sections, submit_button)
    }
}
