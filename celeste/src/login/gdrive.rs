//! The data for a Google Drive Rclone config.
use super::{login_util, ServerType};
use crate::{gtk_util, mpsc::Sender};
use adw::{glib, gtk::Button, prelude::*, ApplicationWindow, EntryRow, MessageDialog};
use libceleste::traits::prelude::*;
use std::{
    cell::RefCell,
    io::Read,
    process::{Command, Stdio},
    rc::Rc,
    thread,
    time::Duration,
};

static DEFAULT_CLIENT_ID: &str =
    "617798216802-gpgajsc7o768ukbdegk5esa3jf6aekgj.apps.googleusercontent.com";
static DEFAULT_CLIENT_SECRET: &str = "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";

#[derive(Clone, Debug, Default)]
pub struct GDriveConfig {
    pub server_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_json: String,
}

impl super::LoginTrait for GDriveConfig {
    fn get_sections(
        window: &ApplicationWindow,
        sender: Sender<Option<ServerType>>,
    ) -> (Vec<EntryRow>, Button) {
        let mut sections: Vec<EntryRow> = vec![];
        let server_name = login_util::server_name_input();
        let submit_button = login_util::submit_button();

        sections.push(server_name.clone());

        submit_button.connect_clicked(glib::clone!(@weak window, @weak server_name => move |_| {
            window.set_sensitive(false);

            let mut process = Command::new("rclone")
                .args(["authorize", "drive", DEFAULT_CLIENT_ID, DEFAULT_CLIENT_SECRET])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let kill_request = Rc::new(RefCell::new(false));

            let dialog = MessageDialog::builder()
                .title(&libceleste::get_title!("Authentication to Google Drive"))
                .heading("Authenticating to Google Drive...")
                .body("Open the link that opened in your browser, and come back once you've finished.")
                .build();
            dialog.add_response("cancel", "Cancel");
            dialog.connect_response(None, glib::clone!(@strong kill_request => move |dialog, resp| {
                if resp != "cancel" {
                    return
                }

                dialog.close();
                *kill_request.get_mut_ref() = true;
            }));
            dialog.show();

            // Run until the process exits or the user clicks 'Cancel'.
            loop {
                // Sleep a little so the UI has a chance to process.
                libceleste::run_in_background(|| thread::sleep(Duration::from_millis(500)));

                // Check if the user clicked cancel.
                if *kill_request.get_ref() {
                    window.set_sensitive(true);
                    return;
                }

                // Otherwise if the command has finished, check if it returned a good exit code and then return it.
                else if let Some(exit_status) = process.try_wait().unwrap() {
                    dialog.close();

                    if !exit_status.success() {
                        let mut stderr_string = String::new();
                        process.stderr.take().unwrap().read_to_string(&mut stderr_string).unwrap();
                        gtk_util::show_codeblock_error("Authentication Error", "There was an issue authenticating to Google Drive", &stderr_string);
                        window.set_sensitive(true);
                        break;
                    } else {
                        let mut stdout_string = String::new();
                        process.stdout.take().unwrap().read_to_string(&mut stdout_string).unwrap();
                        let auth_token = {
                            let lines: Vec<&str> = stdout_string.lines().collect();
                            lines.get(lines.len() - 2).unwrap().to_owned()
                        };

                        sender.send(Some(ServerType::GDrive(GDriveConfig {
                            server_name: server_name.text().to_string(),
                            client_id: DEFAULT_CLIENT_ID.to_string(),
                            client_secret: DEFAULT_CLIENT_SECRET.to_string(),
                            auth_json: auth_token.to_string()
                        })));
                        window.set_sensitive(true);
                        break;
                    }
                }
            }
        }));
        server_name.connect_changed(glib::clone!(@weak submit_button => move |server_name| login_util::check_responses(&[server_name], &submit_button)));

        (sections, submit_button)
    }
}
