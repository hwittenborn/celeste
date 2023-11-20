use crate::util;
use relm4::{
    adw::{self, prelude::*},
    gtk::{ScrolledWindow, TextBuffer, TextView},
};

/// Show an error screen.
pub fn show_error(primary_text: &str, secondary_text: Option<&str>) {
    let dialog = adw::MessageDialog::builder()
        .heading(primary_text)
        .body(secondary_text.unwrap_or(""))
        .build();
    dialog.add_response("", &tr::tr!("Ok"));
    dialog.show();
}

// Show an error screen with a codeblock.
pub fn show_codeblock_error(primary_text: &str, secondary_text: Option<&str>, code: &str) {
    let dialog = adw::MessageDialog::builder()
        .heading(primary_text)
        .body(secondary_text.unwrap_or(""))
        .extra_child(&codeblock(code))
        .resizable(true)
        .build();
    dialog.add_response("", &tr::tr!("Ok"));
    dialog.show();
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
        .css_classes(vec![util::css::SCROLLABLE_CODEBLOCK.to_string()])
        .build()
}
