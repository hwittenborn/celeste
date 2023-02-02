#![feature(let_chains)]
#![feature(arc_unwrap_or_clone)]
#![feature(panic_info_message)]
#![feature(async_closure)]

pub mod about;
pub mod entities;
pub mod gtk_util;
pub mod launch;
pub mod login;
pub mod migrations;
pub mod mpsc;
pub mod rclone;

use adw::{
    gtk::{self, gdk::Display, Align, Box, CssProvider, Label, Orientation, StyleContext},
    prelude::*,
    Application, ApplicationWindow, HeaderBar,
};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    RunGui {},
}

fn main() {
    // Initialize GTK.
    gtk::init().unwrap();

    // Configure Rclone.
    let mut config = libceleste::get_config_dir();
    config.push("rclone.conf");
    librclone::initialize();
    librclone::rpc("config/setpath", json!({ "path": config }).to_string()).unwrap();

    // Load our CSS.
    let provider = CssProvider::new();
    provider.load_from_data(include_bytes!("style.css"));

    StyleContext::add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Get the application.
    let app = Application::builder()
        .application_id(libceleste::APP_ID)
        .build();

    // Due to GTK working in Rust via Rust's FFI, panics don't appear to be able to
    // be captured (this hasn't been confirmed behavior, it's just what I've
    // observed). Panics would like to be captured when they're encountered though,
    // so we relaunch this program in a subprocess and capture any errors from
    // there.
    let cli = Cli::parse();
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::RunGui {} => {
                // Start up the application.
                app.connect_activate(|app| {
                    if app.is_remote() {
                        app.activate();
                        return;
                    }

                    let windows = app.windows();
                    if windows.is_empty() {
                        launch::launch(app);
                    } else {
                        windows.iter().for_each(|window| window.show());
                    }
                });

                app.run_with_args::<&str>(&[]);
            }
        }
    } else {
        // Set `RUST_BACKTRACE` so we get a better backtrace for reporting.
        env::set_var("RUST_BACKTRACE", "1");

        // Run the command and get the stderr, checking for a backtrace.
        let mut command = Command::new(env::args().next().unwrap())
            .arg("run-gui")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout_thread = thread::spawn(move || {
            let mut stdout = String::new();
            let mut stdout_handle = command.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(&mut stdout_handle);

            for line in stdout_reader.lines() {
                let unwrapped_line = line.unwrap();
                println!("{unwrapped_line}");
                stdout.push_str(&unwrapped_line);
                stdout.push('\n');
            }

            stdout
        });
        let stderr_thread = thread::spawn(move || {
            let mut stderr = String::new();
            let mut stderr_handle = command.stderr.as_mut().unwrap();
            let stderr_reader = BufReader::new(&mut stderr_handle);

            for line in stderr_reader.lines() {
                let unwrapped_line = line.unwrap();
                eprintln!("{unwrapped_line}");
                stderr.push_str(&unwrapped_line);
                stderr.push('\n');
            }

            stderr
        });
        let _stdout = stdout_thread.join().unwrap();
        let stderr = stderr_thread.join().unwrap();

        let backtrace = {
            let mut backtrace = String::new();
            let mut backtrace_found = false;

            for line in stderr.lines() {
                if backtrace_found && !line.contains("note: Some details are omitted") {
                    backtrace.push_str(line);
                    backtrace.push('\n');
                } else if line.starts_with("thread 'main' panicked at") {
                    backtrace.push_str(line);
                    backtrace.push('\n');
                    backtrace_found = true;
                }
            }

            backtrace.pop(); // The extra newline at the end.

            if backtrace_found {
                Some(backtrace)
            } else {
                None
            }
        };

        // Show the backtrace in the GUI if one was found.
        if backtrace.is_some() {
            app.connect_startup(move |app| {
                let window = ApplicationWindow::builder()
                    .application(app)
                    .title(&libceleste::get_title!("Unknown Error"))
                    .build();
                let sections = Box::builder()
                    .orientation(Orientation::Vertical)
                    .build();
                sections.append(&HeaderBar::new());
                let error_label = Label::builder()
                    .label("Unknown Error")
                    .halign(Align::Start)
                    .build();
                sections.append(&error_label);

                let error_text = Label::builder()
                    .label("An unknown error has occurred while running. This is an internal issue with Celeste and should be reported.\n\nThe following backtrace may help with debugging the issue - note that it may contain information such as login tokens/keys, so avoid posting the information publically:")
                    .halign(Align::Start)
                    .build();
                sections.append(&error_text);
                sections.append(&gtk_util::codeblock(backtrace.as_ref().unwrap()));

                window.set_content(Some(&sections));
                window.show();
            });

            app.run_with_args::<&str>(&[]);
        }
    }
}
