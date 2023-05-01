use crate::mpsc;
use adw::{
    glib,
    gtk::{Orientation, ScrolledWindow, Separator, TextBuffer, TextView},
    prelude::*,
    MessageDialog,
};

/// Show an error screen.
pub fn show_notification_dialog(primary_text: &str, secondary_text: Option<&str>) {
    let (sender, mut receiver) = mpsc::channel::<()>();
    let mut dialog = MessageDialog::builder()
        .heading(primary_text)
        .modal(true)
        .resizable(true);
    if let Some(text) = secondary_text {
        dialog = dialog.body(text);
    }
    let dialog = dialog.build();
    dialog.add_response("ok", &tr::tr!("Ok"));
    dialog.connect_response(
        None,
        glib::clone!(@strong sender => move |dialog, resp| {
            if ["ok"].contains(&resp) {
                dialog.close();
                sender.send(());
            }
        }),
    );
    dialog.show();
    receiver.recv();
}

/// Show a beta notice dialog.
pub fn show_beta_dialog(feature: &str) {
    show_notification_dialog(
        &tr::tr!("Beta Notice"),
        Some(&tr::tr!("{} support is currently in beta. Usability isn't guaranteed, and support may be dropped in a future release.", feature))
    )
}
/// Show an error screen with a codeblock.
pub fn show_codeblock_error(primary_text: &str, code: &str) {
    let (sender, mut receiver) = mpsc::channel::<()>();
    let dialog = MessageDialog::builder()
        .title(&libceleste::get_title!("{title}"))
        .heading(primary_text)
        .extra_child(&codeblock(code))
        .resizable(true)
        .build();
    dialog.add_response("ok", &tr::tr!("Ok"));
    dialog.connect_response(
        None,
        glib::clone!(@strong sender => move |dialog, resp| {
            if resp != "ok" {
                return;
            }
            dialog.close();
            sender.send(());
        }),
    );
    dialog.show();
    receiver.recv();
}

/// Create a codeblock.
pub fn codeblock(text: &str) -> ScrolledWindow {
    let buffer = TextBuffer::builder().text(text).build();
    let block = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .focusable(false)
        .monospace(true)
        .build();
    ScrolledWindow::builder()
        .child(&block)
        .hexpand(true)
        .vexpand(true)
        .min_content_width(100)
        .min_content_height(100)
        .margin_top(10)
        .css_classes(vec!["celeste-scrollable-codeblock".to_string()])
        .build()
}

/// Get an invisible separator.
pub fn separator() -> Separator {
    Separator::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["spacer".to_string()])
        .build()
}
