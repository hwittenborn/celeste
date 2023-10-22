//! A collection of helper functions for generating login UIs.
use crate::rclone::{self};
use adw::{
    glib,
    gtk::{Align, Button, CheckButton, Label},
    prelude::*,
    EntryRow, PasswordEntryRow,
};

use regex::Regex;
use url::Url;

/// Get the input for the server name.
pub fn server_name_input() -> EntryRow {
    let input = EntryRow::builder().title(&tr::tr!("Name")).build();
    input.connect_changed(|input| {
        let text = input.text();

        // Get a list of already existing config names.
        let existing_remotes: Vec<String> = rclone::get_remotes()
            .iter()
            .map(|config| config.remote_name())
            .collect();

        if existing_remotes.contains(&text.to_string()) {
            input.add_css_class("error");
            input.set_tooltip_text(Some(&tr::tr!("Name already exists.")));
        } else if !Regex::new(r"^[0-9a-zA-Z_.][0-9a-zA-Z_. -]*[0-9a-zA-Z_.-]$").unwrap().is_match(&text) {
            let err_msg = tr::tr!("Invalid name. Names must:\n- Only contain numbers, letters, '_', '-', '.', and spaces\n- Not start with '-' or a space\n- Not end with a space");
            input.add_css_class("error");
            input.set_tooltip_text(Some(&err_msg));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });

    input
}

/// Get the input for the server URL.
pub fn server_url_input(disallow_nextcloud_suffix: bool) -> EntryRow {
    let input = EntryRow::builder().title(&tr::tr!("Server URL")).build();
    input.connect_changed(move |input| {
        let text = input.text();
        let url = Url::parse(&text);

        if let Err(err) = url {
            let err_string = tr::tr!("Invalid server URL ({}).", err);
            input.add_css_class("error");
            input.set_tooltip_text(Some(&err_string));
            return;
        }

        let url = url.unwrap();
        if !url.has_host() {
            input.add_css_class("error");
            input.set_tooltip_text(Some(&tr::tr!("Invalid server URL (no domain specified).")));
        } else if url.password().is_some() {
            input.add_css_class("error");
            input.set_tooltip_text(Some(&tr::tr!(
                "Invalid server URL (password was specified)."
            )));
        } else if !["http", "https"].contains(&url.scheme()) {
            let err_string = tr::tr!(
                "Invalid server URL(unknown server scheme {}).",
                url.scheme()
            );
            input.add_css_class("error");
            input.set_tooltip_text(Some(&err_string));
        } else if disallow_nextcloud_suffix && url.path().contains("/remote.php/") {
            let text_to_remove = Regex::new(r"/remote\.php/.*")
                .unwrap()
                .find(url.path())
                .unwrap()
                .as_str()
                .to_string();
            let err_string = tr::tr!("Don't specify '{}' as part of the URL.", text_to_remove);
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
    EntryRow::builder().title(&tr::tr!("Username")).build()
}

/// Get the input for passwords.
pub fn password_input() -> PasswordEntryRow {
    PasswordEntryRow::builder()
        .title(&tr::tr!("Password"))
        .build()
}

/// Get the input for TOTP/2FA codes.
pub fn totp_input() -> EntryRow {
    let input = EntryRow::builder()
        .title(&tr::tr!("2FA Code"))
        .editable(false)
        .build();
    input.connect_changed(move |input| {
        let text = input.text();

        if text.chars().any(|c| !c.is_numeric()) {
            input.add_css_class("error");
            input.set_tooltip_text(Some(&tr::tr!(
                "The provided 2FA code is invalid (should only contain digits)."
            )));
        } else if text.len() != 6 {
            input.add_css_class("error");
            input.set_tooltip_text(Some(&tr::tr!(
                "The provided 2FA code is invalid (should be 6 digits long)."
            )));
        } else {
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    });
    let check = CheckButton::new();
    check.connect_toggled(glib::clone!(@weak input => move |check| {
        let active = check.is_active();
        input.set_editable(active);

        if !active {
            input.set_text("");
            input.remove_css_class("error");
            input.set_tooltip_text(None);
        }
    }));
    input.add_prefix(&check);
    input
}

/// Get the login button.
pub fn submit_button() -> Button {
    let label = Label::builder().label(&tr::tr!("Log in")).build();
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
        if resp.is_editable() && (resp.has_css_class("error") || resp.text().is_empty()) {
            no_errors = false;
        }
    }

    submit_button.set_sensitive(no_errors);
}
