use std::convert::identity;

use adw::prelude::*;
use relm4::{component::{AsyncComponent, SimpleAsyncComponent, AsyncComponentSender, AsyncComponentParts, AsyncController, AsyncComponentController}, prelude::*};

use crate::login::{LoginModel, LoginMsg};

pub enum LauchMsg {
    NewLogin,
    OpenRequest,
    CloseRequest,
}

pub struct LaunchModel {
    hide_on_close: bool,
    visible: bool,
    login: AsyncController<LoginModel>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for LaunchModel {
    type Input = ();
    type Output = ();
    type Init = ();

    view! {
        adw::ApplicationWindow {
            #[watch]
            set_hide_on_close: model.hide_on_close,
            #[watch]
            set_visible: model.visible,

            adw::HeaderBar {}
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        println!("INITTED!");
        relm4::tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let login = LoginModel::builder()
            .transient_for(root.clone())
            .launch(())
            .forward(sender.input_sender(), identity);
        let model = Self {
            hide_on_close: false,
            visible: false,
            login,
        };
        let widgets = view_output!();

        model.login.emit(LoginMsg::Open);

        AsyncComponentParts { model, widgets }
    }
}
