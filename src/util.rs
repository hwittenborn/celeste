use gtk::{Application, ApplicationWindow, Label, prelude::*};

// Create a new application instance.
pub fn application() -> Application {
    Application::builder()                                                                                
        .application_id("com.hunterwittenborn.cliff")
        .build()
}

// Create a plain text window.
pub fn create_text_window(app: &gtk::Application, title: &str, text: &str) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(title)
        .build();

    let label = Label::builder()
        .label(text)
        .margin_top(10)
        .margin_end(10)
        .margin_bottom(10)
        .margin_start(10)
        .build();

    window.set_child(Some(&label));
    window.show();
    window
}
