use gtk3::{glib, prelude::*, Menu, MenuItem};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::sync::Mutex;
use zbus::blocking::Connection;

lazy_static::lazy_static! {
    static ref CLOSE_REQUEST: Mutex<bool> = Mutex::new(false);
    static ref SYNC_ICON_REQUEST: Mutex<bool> = Mutex::new(false);
    static ref WARNING_ICON_REQUEST: Mutex<bool> = Mutex::new(false);
    static ref DONE_ICON_REQUEST: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_STATUS: Mutex<String> = Mutex::new(String::new());
}

struct TrayIcon;

#[zbus::dbus_interface(name = "com.hunterwittenborn.Celeste.Tray")]
impl TrayIcon {
    async fn close(&self) {
        *(*CLOSE_REQUEST).lock().unwrap() = true;
    }

    async fn update_status(&self, status: &str) {
        *(*CURRENT_STATUS).lock().unwrap() = status.to_string();
    }

    async fn set_syncing_icon(&self) {
        *(*SYNC_ICON_REQUEST).lock().unwrap() = true;
    }

    async fn set_warning_icon(&self) {
        *(*WARNING_ICON_REQUEST).lock().unwrap() = true;
    }

    async fn set_done_icon(&self) {
        *(*DONE_ICON_REQUEST).lock().unwrap() = true;
    }
}

fn main() {
    gtk3::init().unwrap();

    // The indicator.
    let mut indicator = AppIndicator::new(
        "Celeste",
        "com.hunterwittenborn.Celeste.CelesteTrayLoading-symbolic",
    );
    indicator.set_status(AppIndicatorStatus::Active);

    let mut menu = Menu::new();
    let menu_sync_status = MenuItem::builder()
        .label(&tr::tr!("Awaiting sync checks..."))
        .sensitive(false)
        .build();
    let menu_open = MenuItem::builder().label(&tr::tr!("Open")).build();
    let menu_quit = MenuItem::builder().label(&tr::tr!("Quit")).build();
    menu.append(&menu_sync_status);
    menu.append(&menu_open);
    menu.append(&menu_quit);
    indicator.set_menu(&mut menu);

    // Our DBus connection to receive messages from the main application.
    let connection = Connection::session().unwrap();
    connection
        .object_server()
        .at(libceleste::DBUS_TRAY_OBJECT, TrayIcon)
        .unwrap();
    connection.request_name(libceleste::TRAY_ID).unwrap();

    // Helper function to call a Celeste-side DBus function.
    let call_fn = glib::clone!(@strong connection => move |func: &str| {
        connection.call_method(
            Some(libceleste::DBUS_APP_ID),
            libceleste::DBUS_APP_OBJECT,
            Some(libceleste::DBUS_APP_ID),
            func,
            &()
        )
    });

    // Button connections.
    menu_open.connect_activate(glib::clone!(@strong call_fn => move |_| {
        call_fn("Open").unwrap();
    }));
    menu_quit.connect_activate(|_| {
        *(*CLOSE_REQUEST).lock().unwrap() = true;
    });

    // Start up the application.
    menu.show_all();

    loop {
        #[allow(clippy::if_same_then_else)]
        gtk3::main_iteration_do(false);

        let status = (*(*CURRENT_STATUS).lock().unwrap()).clone();
        indicator.set_title(&status);
        menu_sync_status.set_label(&status);

        if *(*SYNC_ICON_REQUEST).lock().unwrap() {
            indicator.set_icon("com.hunterwittenborn.Celeste.CelesteTraySyncing-symbolic");
        } else if *(*DONE_ICON_REQUEST).lock().unwrap() {
            indicator.set_icon("com.hunterwittenborn.Celeste.CelesteTrayDone-symbolic");
        } else if *(*WARNING_ICON_REQUEST).lock().unwrap() {
            indicator.set_icon("com.hunterwittenborn.Celeste.CelesteTrayWarning-symbolic");
        }

        *(*SYNC_ICON_REQUEST).lock().unwrap() = false;
        *(*WARNING_ICON_REQUEST).lock().unwrap() = false;
        *(*DONE_ICON_REQUEST).lock().unwrap() = false;

        if *(*CLOSE_REQUEST).lock().unwrap() {
            // Set up the quit label.
            menu_quit.set_sensitive(false);
            menu_quit.set_label(&tr::tr!("Quitting..."));

            // Notify the tray icon to close.
            // I'm not sure when this can fail, so output an error if one is received.
            if let Err(err) = call_fn("Close") {
                hw_msg::warningln!(
                    "Got error while sending close request to main application: '{err}'."
                );
            };

            // And then quit the application.
            break;
        }
    }
}
