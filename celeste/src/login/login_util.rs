//! A collection of helper functions for generating login UIs.
use crate::rclone::{self};
use adw::{
    gtk::{Align, Button, Label},
    prelude::*,
    EntryRow, PasswordEntryRow,
};

use url::Url;

/// Get the input for the server name.
pub fn server_name_input() -> EntryRow {
    let input = EntryRow::builder().title("Server Name").build();
    input.connect_changed(|input| {
        let text = input.text();

        // Get a list of already existing config names.
        let existing_remotes: Vec<String> = rclone::get_remotes()
            .iter()
            .map(|config| config.remote_name())
            .collect();

        if existing_remotes.contains(&text.to_string()) {
            input.add_css_class("error");
            input.set_tooltip_text(Some("Server name already exists."));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });

    input
}

/// Get the input for the server URL.
pub fn server_url_input() -> EntryRow {
    let input = EntryRow::builder().title("Server URL").build();
    input.connect_changed(|input| {
        let text = input.text();
        let url = Url::parse(&text);

        if let Err(err) = url {
            let err_string = format!("Invalid server URL ({err}).");
            input.add_css_class("error");
            input.set_tooltip_text(Some(&err_string));
        } else if !url.as_ref().unwrap().has_host() {
            input.add_css_class("error");
            input.set_tooltip_text(Some("Invalid server URL (no domain specified)."));
        } else if url.as_ref().unwrap().password().is_some() {
            input.add_css_class("error");
            input.set_tooltip_text(Some("Invalid server URL (password was specified."));
        } else if !["http", "https"].contains(&url.as_ref().unwrap().scheme()) {
            let err_string = format!(
                "Invalid server URL(unknown server scheme {}).",
                url.as_ref().unwrap().scheme()
            );
            input.add_css_class("error");
            input.set_tooltip_text(Some(&err_string));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });
    input
}

/// Get the input for usernames.
pub fn username_input() -> EntryRow {
    let input = EntryRow::builder().title("Username").build();
    input.connect_changed(|input| {
        let _text = input.text();

        if input.text().contains(':') {
            input.add_css_class("error");
            input.set_tooltip_text(Some("Username isn't allowed to contain ':'."));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });
    input
}

/// Get the input for passwords.
pub fn password_input() -> PasswordEntryRow {
    let input = PasswordEntryRow::builder().title("Password").build();
    input.connect_changed(|input| {
        let _text = input.text();

        if input.text().contains(':') {
            input.add_css_class("error");
            input.set_tooltip_text(Some("Password isn't allowed to contain ':'."));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });
    input
}

/// Get the login button.
pub fn submit_button() -> Button {
    let label = Label::builder().label("Log in").build();
    let button = Button::builder()
        .child(&label)
        .halign(Align::End)
        .margin_top(10)
        .css_classes(vec!["login-window-submit-button".to_string()])
        .build();
    // Grey out the button initially so it can't be until items are validated.
    button.set_sensitive(false);
    button
}

/// Grey out the password button if any of the specified fields have errors or
/// are empty. This ignores any entries that aren't sensitive.
pub fn check_responses(responses: &[&EntryRow], submit_button: &Button) {
    let mut no_errors = true;

    for resp in responses {
        if resp.is_sensitive() && (resp.has_css_class("error") || resp.text().is_empty()) {
            no_errors = false;
        }
    }

    submit_button.set_sensitive(no_errors);
}
