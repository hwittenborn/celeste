//! Functionality for logging into a server.
use crate::{gtk_util, rclone, util};
use adw::{prelude::*, Application, ApplicationWindow};
use nix::{sys::signal::{self, Signal}, unistd::Pid};
use regex::Regex;
use relm4::{
    component::{AsyncComponentParts, AsyncComponentSender, SimpleAsyncComponent},
    prelude::*,
};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};
use sea_orm::DatabaseConnection;
use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
    io::{BufRead, BufReader},
    net::SocketAddr,
    process::{Child, Command, Stdio},
    rc::Rc,
    str::FromStr,
    sync::{Arc, LazyLock, Mutex},
    thread, time::Duration,
};
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};
use tera::{Context, Tera};
use tokio::sync::mpsc::{self, Sender, Receiver};
use url::Url;
use warp::{
    http::{header, Response},
    Filter,
};

static GOOGLE_DRIVE_AUTH_HTML: &str = include_str!("html/google-drive.tera.html");
static GOOGLE_DRIVE_PNG: &[u8] = include_bytes!("images/google-drive.png");
static GOOGLE_DRIVE_SIGNIN_PNG: &[u8] = include_bytes!("images/google-signin.png");

fn show_error(model: &LoginModel, field: &LoginField) {
    let mut borrow = model.errors.borrow_mut();
    let items = borrow.get_mut(field).unwrap();

    // TODO: We should use `Alert` from `relm4_components` for this, but their
    // component currently isn't flexible enough for our needs.
    gtk_util::show_error(&items.0, Some(&items.1));
}

/// Spawn the Google Drive authentication server.
///
/// Returns the server address, and a [`Sender`] that can be used to
/// stop the server.
async fn spawn_drive_auth_server(rclone_url: &str) -> (SocketAddr, Sender<()>) {
    let rclone_url = rclone_url.to_string();
    let root = warp::path::end().map(move || {
        let mut context = Context::new();
        context.insert("rclone_url", &rclone_url);
        let html = Tera::one_off(GOOGLE_DRIVE_AUTH_HTML, &context, true).unwrap();
        warp::reply::html(html)
    });
    let drive_png = warp::path!("google-drive.png").map(|| {
        Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(GOOGLE_DRIVE_PNG)
    });
    let drive_signin_png = warp::path!("google-signin.png").map(|| {
        Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(GOOGLE_DRIVE_SIGNIN_PNG)
    });

    let routes = warp::get().and(root.or(drive_png).or(drive_signin_png));

    let (tx, mut rx) = mpsc::channel(1);
    let (addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(SocketAddr::from_str("127.0.0.1:0").unwrap(), async move {
            rx.recv().await.unwrap()
        });
    tokio::spawn(server);

    (addr, tx)
}

/// Get an authentication token for the given provider. Returns [`Err`] if
/// unable to.
/// 
/// `rx` is an [`mpsc::Receiver`] to cancel authentication requests with.
async fn get_token(provider: Provider, mut rx: mpsc::Receiver<()>) -> Result<String, LoginCommandErr> {
    let (client_id, client_secret) = match provider {
        Provider::Dropbox => ("hke0fgr43viaq03", "o4cpx8trcnneq7a"),
        Provider::GoogleDrive => (
            "617798216802-gpgajsc7o768ukbdegk5esa3jf6aekgj.apps.googleusercontent.com",
            "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",
        ),
        Provider::PCloud => ("KRzpo46NKb7", "g10qvqgWR85lSvEQWlIqCmPYIhwX"),
        _ => panic!("An invalid provider was entered"),
    };

    let rclone_args = [
        "authorize",
        provider.rclone_type(),
        client_id,
        client_secret,
        "--auth-no-open-browser",
    ];

    // Spawn the authentication process, and continuously read it's stdout and stdin
    // into strings.
    let rclone_stdout: Arc<Mutex<String>> = Arc::default();
    let rclone_stderr: Arc<Mutex<String>> = Arc::default();

    let mut process = Command::new("rclone")
        .args(&rclone_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let process_stdout = process.stdout.take().unwrap();
    let process_stderr = process.stderr.take().unwrap();
    let stdout_thread = thread::spawn(glib::clone!(@strong rclone_stdout => move || {
        let reader = BufReader::new(process_stdout);
        for line in reader.lines() {
            let mut string = line.unwrap();
            string.push('\n');
            rclone_stdout.lock().unwrap().push_str(&string);
        }
    }));
    let stderr_thread = thread::spawn(glib::clone!(@strong rclone_stderr => move || {
        let reader = BufReader::new(process_stderr);
        for line in reader.lines() {
            let mut string = line.unwrap();
            string.push('\n');
            rclone_stderr.lock().unwrap().push_str(&string);
        }
    }));

    // Get the URL rclone will use for authentication.
    let rclone_url = loop {
        // If the rclone process has aborted already, then it failed before being able to get us a URL and we need to let the user know.
        if process.try_wait().unwrap().is_some() {
            break Err(rclone_stderr.lock().unwrap().to_string())
        }

        // Otherwise check if the URL line has been printed in stdout or stderr. Currently in rclone, this involves checking for a URL at the end of a line.
        let output = format!(
            "{}\n{}",
            rclone_stdout.lock().unwrap(),
            rclone_stderr.lock().unwrap()
        );
        let maybe_url = output.lines()
            .find(|line| line.contains("http://127.0.0.1:53682/auth"))
            .map(|line| line.split_whitespace().last().unwrap().to_owned());

        if let Some(url) = maybe_url {
            break Ok(url)
        }
    }
    .map_err(|err| LoginCommandErr::AuthServer(err))?;

    // Present the authentication request to the user.
    //
    // Google Drive has requirements for our app to show a Google Drive logo, so
    // handle that here.
    let (addr, maybe_killer) = if provider == Provider::GoogleDrive {
        let (addr, killer) = spawn_drive_auth_server(&rclone_url).await;
        (format!("http://{addr}"), Some(killer))
    } else {
        (rclone_url.to_string(), None)
    };
    open::that(&addr).unwrap();

    // Get the token, returning an error if we couldn't get it.
    let token = relm4::spawn_blocking(move || loop {
        // Check if the user is cancelling the request.
        if rx.try_recv().is_ok() {
            // Kill the rclone process so we can use it in subsequent requests.
            let pid = Pid::from_raw(process.id().try_into().unwrap());
            signal::kill(pid, Signal::SIGINT).unwrap();
            break Err(LoginCommandErr::Cancelled);
        // Otherwise if the command finished, check if it returned a good exit code and then return the token.
        } else if let Some(exit_status) = process.try_wait().unwrap() {
            if !exit_status.success() {
                break Err(LoginCommandErr::Token(rclone_stderr.lock().unwrap().to_string()))
            } else {
                let token = rclone_stdout.lock()
                    .unwrap()
                    .lines()
                    .rev()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                break Ok(token)
            }
        }
    })
    .await
    .unwrap();

    // Kill the webserver if we had started it up.
    if let Some(killer) = maybe_killer {
        killer.send(()).await.unwrap();
    }

    Ok(token?)
}

#[relm4::widget_template]
impl WidgetTemplate for WarningButton {
    view! {
        gtk::Button {
            add_css_class: "flat",
            set_icon_name: relm4_icons::icon_name::WARNING,
            set_valign: gtk::Align::Center
        }
    }
}

#[derive(Clone, Debug, Default, EnumIter, EnumString, IntoStaticStr, PartialEq, strum::Display)]
pub enum Provider {
    #[default]
    Dropbox,
    #[strum(serialize = "Google Drive")]
    GoogleDrive,
    Nextcloud,
    Owncloud,
    #[strum(serialize = "pCloud")]
    PCloud,
    #[strum(serialize = "Proton Drive")]
    ProtonDrive,
    WebDav,
}

impl Provider {
    /// The name rclone uses to identity this remote type.
    fn rclone_type(&self) -> &'static str {
        match self {
            Self::Dropbox => "dropbox",
            Self::GoogleDrive => "drive",
            Self::Nextcloud | Self::Owncloud | Self::WebDav => "webdav",
            Self::PCloud => "pcloud",
            Self::ProtonDrive => "protondrive",
        }
    }
}

#[derive(Clone, Debug)]
pub enum LoginMsg {
    /// Open the login window.
    Open,
    /// Set the provider we want to log in with.
    #[doc(hidden)]
    SetProvider(Provider),
    /// Check that the inputs provided by the user are valid.
    #[doc(hidden)]
    CheckInputs,
    /// Show an error from clicking the warning button on a login field.
    #[doc(hidden)]
    ShowFieldError(LoginField),
    /// Get a token for a service that needs it.
    #[doc(hidden)]
    Authenticate,
    /// Cancel an active authentication session from [`Self::Authenticate`].
    #[doc(hidden)]
    CancelAuthenticate,
}

#[derive(Debug)]
pub enum LoginResponse {
    // The user closed the window without logging in.
    NoLogin,
    // The remote name in the rclone config for the new login.
    NewLogin(String)
}

/// The login fields that we need to check. We use this in [`LoginModel`] below.
#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
enum LoginField {
    Name,
    Url,
    Username,
    Password,
    Totp,
}

#[derive(Debug)]
pub enum LoginCommandErr {
    /// The authentication request was cancelled.
    Cancelled,
    /// An error from starting up the rclone authorization server. Contains the
    /// stderr of `rclone authorize`.
    AuthServer(String),
    /// An error from obtaining a token from the rclone authorization server.
    /// Contains the stderr of `rclone authorize`.
    Token(String),
}

/// The type we use to store errors. The values are a tuple of (title, subtitle)
/// messages to pass to a message window.
type Errors = HashMap<LoginField, (String, String)>;

pub struct LoginModel {
    visible: bool,
    provider: Provider,
    errors: Rc<RefCell<Errors>>,
    /// An [`Sender`] to use when cancelling authentication requests from [`Self::auth`]. It gets set to [`Some`] at the start of an authentication request from [`LoginMsg::Authenticate`].
    auth_sender: Option<Sender<()>>,
    /// The [`Alert`] component we use for showing errors from [`Self::errors`].
    alert: Controller<Alert>,
    /// The [`Alert`] component we use to notify the user of web browser
    /// authentication.
    auth: Controller<Alert>,
}

#[relm4::component(async, pub)]
impl AsyncComponent for LoginModel {
    type Input = LoginMsg;
    type Output = LoginResponse;
    type CommandOutput = Result<String, LoginCommandErr>;
    type Init = ();

    view! {
        #[name(window)]
        ApplicationWindow {
            set_title: Some(&util::get_title!("Log in")),
            set_default_width: 400,
            add_css_class: "celeste-global-padding",
            #[watch]
            set_visible: model.visible,
            // When hiding/showing different entry widgets, we may end up with
            // extra padding on the bottom of the window. This resets the
            // window height to our widget size on each render.
            #[watch]
            set_default_size: (window.width(), -1),

             gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    add_css_class: "boxed-list",

                    adw::ComboRow {
                        set_title: &tr::tr!("Server Type"),

                        #[wrap(Some)]
                        set_model = &gtk::StringList {
                            #[iterate]
                            append: Provider::iter().map(|provider| provider.into())
                        },

                        connect_selected_item_notify[sender] => move |row| {
                            let string_list: gtk::StringList = row.model().unwrap().downcast().unwrap();
                            let selected = string_list.string(row.selected()).unwrap().to_string();
                            let provider = Provider::from_str(&selected).unwrap();
                            sender.input(LoginMsg::SetProvider(provider));
                        }
                    },

                    #[name(name_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Name"),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Name).unwrap().0.is_empty(),
                            connect_clicked => LoginMsg::ShowFieldError(LoginField::Name)
                        },

                        connect_changed[errors = model.errors.clone(), sender] => move |name_input| {
                            let name = name_input.text().to_string();

                            // Get a list of already existing config names.
                            let existing_remotes: Vec<String> = rclone::get_remotes()
                                .iter()
                                .map(|config| config.remote_name())
                                .collect();

                            // Check that the new specified remote is valid.
                            static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-zA-Z_.][0-9a-zA-Z_. -]*[0-9a-zA-Z_.-]$").unwrap());

                            let mut err_msg = None;

                            if name.is_empty() {
                                err_msg = None;
                            } else if existing_remotes.contains(&name) {
                                err_msg = (tr::tr!("Name already exists"), String::new()).into();
                            } else if !NAME_REGEX.is_match(&name) {
                                err_msg = (
                                    tr::tr!("Invalid server name"),
                                    format!(
                                        "{}\n- {}\n- {}\n- {}",
                                        tr::tr!("Server names must:"),
                                        tr::tr!("Only contain numbers, letters, underscores, hyphens, periods, and spaces"),
                                        tr::tr!("Not start with a hyphen/space"),
                                        tr::tr!("Not end with a space")
                                    )
                                ).into();
                            }

                            if let Some(msg) = err_msg {
                                *errors.borrow_mut().get_mut(&LoginField::Name).unwrap() = msg;
                                name_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = errors.borrow_mut();
                                let items = borrow.get_mut(&LoginField::Name).unwrap();
                                items.0.clear();
                                items.1.clear();
                                name_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        },
                    },

                    #[name(url_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Server URL"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::WebDav),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Url).unwrap().0.is_empty(),
                            connect_clicked => LoginMsg::ShowFieldError(LoginField::Url)
                        },
                        connect_changed[errors = model.errors.clone(), provider = model.provider.clone(), sender] => move |url_input| {
                            let mut err_msg = None;
                            let maybe_url = Url::parse(&url_input.text());

                            if url_input.text().is_empty() {
                                err_msg = None;
                            } else if let Ok(url) = maybe_url {
                                if matches!(provider, Provider::Nextcloud | Provider::Owncloud) && url.path().contains("/remote.php/") {
                                    static REMOTE_PHP_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/remote\.php/.*").unwrap());
                                    let invalid_url_segment = REMOTE_PHP_REGEX.find(url.path())
                                        .unwrap()
                                        .as_str()
                                        .to_string();
                                    err_msg = (
                                        tr::tr!("Invalid server URL"),
                                        tr::tr!("Don't specify '{invalid_url_segment}' as part of the URL"),
                                    ).into();
                                }
                            } else {
                                err_msg = (
                                    tr::tr!("Invalid server URL"),
                                    tr::tr!("Error: {}.", maybe_url.unwrap_err())
                                ).into();
                            }

                            if let Some(msg) = err_msg {
                                *errors.borrow_mut().get_mut(&LoginField::Url).unwrap() = msg;
                                url_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = errors.borrow_mut();
                                let items = borrow.get_mut(&LoginField::Url).unwrap();
                                items.0.clear();
                                items.1.clear();
                                url_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        }
                    },

                    #[name(username_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Username"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav),
                        connect_changed => LoginMsg::CheckInputs,
                    },

                    #[name(password_input)]
                    adw::PasswordEntryRow {
                        set_title: &tr::tr!("Password"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav),
                        connect_changed => LoginMsg::CheckInputs,
                    },

                    #[name(totp_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("2FA Code"),
                        set_editable: false,
                        #[watch]
                        set_visible: matches!(model.provider, Provider::ProtonDrive),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: totp_input_checkmark.is_active() && !model.errors.borrow().get(&LoginField::Totp).unwrap().0.is_empty(),
                            connect_clicked => LoginMsg::ShowFieldError(LoginField::Totp)
                        },
                        connect_changed[errors = model.errors.clone(), sender] => move |totp_input| {
                            let totp = totp_input.text().to_string();
                            let mut err_msg = None;

                            if totp.is_empty() {
                                err_msg = None;
                            } else if totp.chars().any(|c| !c.is_numeric()) {
                                err_msg = (
                                    tr::tr!("Invalid 2FA code"),
                                    tr::tr!("The 2FA code should only contain digits")
                                ).into();
                            } else if totp.len() != 6 {
                                err_msg = (
                                    tr::tr!("Invalid 2FA code"),
                                    tr::tr!("The 2FA code should be 6 digits long")
                                ).into();
                            }

                            if let Some(msg) = err_msg {
                                *errors.borrow_mut().get_mut(&LoginField::Totp).unwrap() = msg;
                                totp_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = errors.borrow_mut();
                                let items = borrow.get_mut(&LoginField::Totp).unwrap();
                                items.0.clear();
                                items.1.clear();
                                totp_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        },

                        #[name(totp_input_checkmark)]
                        add_prefix = &gtk::CheckButton {
                            connect_toggled[sender, totp_input] => move |check| {
                                let active = check.is_active();
                                totp_input.set_editable(active);

                                if !active {
                                    totp_input.set_text("");
                                    totp_input.remove_css_class(util::css::ERROR);
                                }
                            }
                        }
                    }
                },

                #[name(login_button)]
                gtk::Button {
                    set_label: &tr::tr!("Log in"),
                    set_halign: gtk::Align::End,
                    set_margin_top: 10,
                    add_css_class: "login-window-submit-button",
                    set_sensitive: false,

                    connect_clicked => LoginMsg::Authenticate
                }
             }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let alert = Alert::builder()
            .transient_for(root.clone())
            .launch(AlertSettings {
                confirm_label: Some(tr::tr!("Ok")),
                ..Default::default()
            })
            .connect_receiver(|_, _| {});
        let auth = Alert::builder()
            .transient_for(root.clone())
            .launch(AlertSettings {
                text: "".to_string(),
                secondary_text: Some(tr::tr!("Follow the link that opened in your browser, and come back once you've finished")),
                cancel_label: Some(tr::tr!("Cancel")),
                ..Default::default()
            })
            .forward(sender.input_sender(), |_| LoginMsg::CancelAuthenticate);

        let mut model = Self {
            visible: false,
            provider: Provider::default(),
            errors: Rc::default(),
            auth_sender: None,
            alert,
            auth,
        };
        for field in LoginField::iter() {
            model.errors.borrow_mut().insert(field, Default::default());
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        // Reset all the input widgets we use to be empty. We do this when
        // opening/re-opening the window or switching providers.
        let reset_widgets = || {
            widgets.name_input.set_text("");
            widgets.url_input.set_text("");
            widgets.username_input.set_text("");
            widgets.password_input.set_text("");
            widgets.totp_input_checkmark.set_active(false);
        };

        match message {
            LoginMsg::Open => {
                reset_widgets();
                sender.input(LoginMsg::SetProvider(Provider::default()));
                self.visible = true;
            }
            LoginMsg::SetProvider(provider) => {
                reset_widgets();
                self.provider = provider;
            }
            LoginMsg::CheckInputs => {
                // Disable the login button if any current input fields are empty or contain
                // errors.
                let mut sensitive = true;
                let inputs: Vec<adw::EntryRow> = vec![
                    widgets.name_input.clone(),
                    widgets.url_input.clone(),
                    widgets.username_input.clone(),
                    widgets.password_input.clone().into(),
                ];

                for input in inputs {
                    if input.is_visible() {
                        if input.text().is_empty() || input.has_css_class(util::css::ERROR) {
                            sensitive = false;
                        }
                    }
                }

                // We have to check the TOTP field separately, as it contains a checkmark
                // toggle.
                if widgets.totp_input.is_visible() && widgets.totp_input_checkmark.is_active() {
                    if widgets.totp_input.text().is_empty()
                        || widgets.totp_input.has_css_class(util::css::ERROR)
                    {
                        sensitive = false;
                    }
                }

                widgets.login_button.set_sensitive(sensitive);
            }
            LoginMsg::ShowFieldError(field) => {
                let mut errors_ref = self.errors.borrow_mut();
                let error_items = errors_ref.get_mut(&field).unwrap();

                let mut alert_state = self.alert.state().get_mut();
                let mut settings = &mut alert_state.model.settings;

                settings.text = error_items.0.clone();
                settings.secondary_text = Some(error_items.1.clone());
                self.alert.emit(AlertMsg::Show);
            }
            LoginMsg::Authenticate => {
                todo!("We have to make this function only start up the token server when using a required remote. I'm too tired to look at this function right now so we're out.");
                root.set_sensitive(false);
                let (tx, rx) = mpsc::channel(1);

                self.auth.state().get_mut().model.settings.text = tr::tr!("Logging into {}...", self.provider);
                self.auth.emit(AlertMsg::Show);

                let provider = self.provider.clone();
                sender.oneshot_command(async {
                    let token = get_token(provider, rx).await?;
                    todo!("Gotta set up the rclone config now. Here we go D:");
                });
                self.auth_sender = Some(tx);
            }
            LoginMsg::CancelAuthenticate => {
                self.auth_sender.take().unwrap().send(()).await.unwrap();
            }
        }

        self.update_view(widgets, sender);
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.auth.emit(AlertMsg::Hide);

        match message {
            Ok(remote_name) => sender.output(LoginResponse::NewLogin(remote_name)).unwrap(),
            Err(err) => match err {
                // Everything that needs to be handled is done in the code above and below this `match` statement.
                LoginCommandErr::Cancelled => (),
                // TODO: Both of these should use Relm4 components, but we're gonna see if we can
                // get [`Alert`] from `relm4_components` to use `adw::MessageDialog` first.
                LoginCommandErr::AuthServer(err) => gtk_util::show_codeblock_error(
                    &tr::tr!("Unable to start authentication server"),
                    Some(&tr::tr!(
                        "More information about the error is included below"
                    )),
                    &err,
                ),
                LoginCommandErr::Token(err) => gtk_util::show_codeblock_error(
                    &tr::tr!("Unable to obtain token"),
                    Some(&tr::tr!(
                        "More information about the error is included below"
                    )),
                    &err,
                ),
            },
        }

        root.set_sensitive(true);
    }
}
