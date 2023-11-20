//! Functionality for logging into a server.
use crate::{gtk_util, rclone, util};
use adw::{prelude::*, Application, ApplicationWindow};
use regex::Regex;
use relm4::{
    component::{AsyncComponentParts, AsyncComponentSender, SimpleAsyncComponent},
    prelude::*,
};
use relm4_components::alert::{Alert, AlertMsg, AlertSettings};
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
    thread,
};
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};
use tera::{Context, Tera};
use tokio::sync::oneshot;
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
/// Returns the server address, and a [`oneshot::Sender`] that can be used to
/// stop the server.
async fn spawn_drive_auth_server(rclone_url: &str) -> (SocketAddr, oneshot::Sender<()>) {
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

    let (tx, rx) = oneshot::channel();
    let (addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(SocketAddr::from_str("127.0.0.1:0").unwrap(), async {
            rx.await.unwrap()
        });
    tokio::spawn(server);

    (addr, tx)
}

/// Get an authentication token for the given provider. Returns [`Err`] if unable to.
async fn get_token(provider: Provider) -> Result<String, LoginCommandErr> {
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
    let rclone_url = relm4::spawn_blocking(
        glib::clone!(@strong rclone_stdout, @strong rclone_stderr => move || loop {
            // If the rclone process has aborted already, then it failed before being able to get us a URL and we need to let the user know.
            if process.try_wait().unwrap().is_some() {
                return Err(rclone_stderr.lock().unwrap().to_string())
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
        }),
    )
    .await
    .unwrap()
    .map_err(|err| LoginCommandErr::AuthServer(err))?;

    // Present the authentication request to the user.
    //
    // Google Drive has requirements for our app to show a Google Drive logo, so
    // handle that here.
    let (addr, killer) = if provider == Provider::GoogleDrive {
        let (addr, killer) = spawn_drive_auth_server(&rclone_url).await;
        (addr.to_string(), Some(killer))
    } else {
        (rclone_url, None)
    };
    open::that(&format!("http://{addr}")).unwrap();
    todo!("Wait until we get a token, and then return it from this function")
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

#[derive(Clone, Debug, Default, EnumIter, EnumString, IntoStaticStr, PartialEq)]
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
    Open,
    #[doc(hidden)]
    SetProvider(Provider),
    #[doc(hidden)]
    CheckInputs,
    #[doc(hidden)]
    Authenticate,
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
    /// An error from starting up the rclone authorization server. Contains the stderr of `rclone authorize`.
    AuthServer(String),
}

/// The type we use to store errors. The values are a tuple of (title, subtitle)
/// messages to pass to a message window.
type Errors = HashMap<LoginField, (String, String)>;

#[derive(Clone, Default)]
pub struct LoginModel {
    visible: bool,
    provider: Provider,
    errors: Rc<RefCell<Errors>>,
}

#[relm4::component(async, pub)]
impl AsyncComponent for LoginModel {
    type Input = LoginMsg;
    type Output = ();
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
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Name)
                        },

                        connect_changed[model, sender] => move |name_input| {
                            let name = name_input.text().to_string();

                            // Get a list of already existing config names.
                            let existing_remotes: Vec<String> = rclone::get_remotes()
                                .iter()
                                .map(|config| config.remote_name())
                                .collect();

                            // Check that the new specified remote is valid.
                            static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-zA-Z_.][0-9a-zA-Z_. -]*[0-9a-zA-Z_.-]$").unwrap());

                            let mut err_msg = None;
                            if existing_remotes.contains(&name) {
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
                                *model.errors.borrow_mut().get_mut(&LoginField::Name).unwrap() = msg;
                                name_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = model.errors.borrow_mut();
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
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Url)
                        },
                        connect_changed[model, sender] => move |url_input| {
                            let mut err_msg = None;
                            let maybe_url = Url::parse(&url_input.text());

                            if let Ok(url) = maybe_url {
                                if matches!(model.provider, Provider::Nextcloud | Provider::Owncloud) && url.path().contains("/remote.php/") {
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
                                *model.errors.borrow_mut().get_mut(&LoginField::Url).unwrap() = msg;
                                url_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = model.errors.borrow_mut();
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
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Totp)
                        },
                        connect_changed[model, sender] => move |totp_input| {
                            let totp = totp_input.text().to_string();
                            let mut err_msg = None;

                            if totp.chars().any(|c| !c.is_numeric()) {
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
                                *model.errors.borrow_mut().get_mut(&LoginField::Totp).unwrap() = msg;
                                totp_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = model.errors.borrow_mut();
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
        let mut model = Self::default();
        for field in LoginField::iter() {
            model.errors.borrow_mut().insert(field, Default::default());
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    fn post_view() {
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

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>, _root: &Self::Root) {
        // We have to clone the provider in order to use it in the
        // `LoginMsg::Authenticate` match below.
        let provider_clone = self.provider.clone();

        match message {
            LoginMsg::Open => self.visible = true,
            LoginMsg::SetProvider(provider) => self.provider = provider,
            // This is handled in `pre_view` above. Preferrably it would be
            // done here, but we can't access the struct's widgets here.
            LoginMsg::CheckInputs => (),
            LoginMsg::Authenticate => sender.oneshot_command(async {
                if matches!(
                    provider_clone,
                    Provider::Dropbox | Provider::GoogleDrive | Provider::PCloud
                ) {
                    todo!("Show a window telling the user to look in their browser");
                    let token = get_token(provider_clone).await?;
                }

                todo!()
            }),
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            Ok(_) => todo!("GOTTA DO SOMETHING WITH THAT STUFF"),
            Err(err) => match err {
                LoginCommandErr::AuthServer(err) => gtk_util::show_codeblock_error(
                    &tr::tr!("Unable to start authentication server"),
                    Some(&tr::tr!("More information about the error is included below")),
                    &err
                )
            }
        }
    }
}
