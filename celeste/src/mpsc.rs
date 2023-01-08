use adw::gtk::glib::MainContext;
use std::fmt::Debug;
use tokio::sync::mpsc::{self as tokio_mpsc, Receiver as TokioReceiver, Sender as TokioSender};

/// Sends values to the associated [`Receiver`].
#[derive(Clone)]
pub struct Sender<T> {
    sender: TokioSender<T>,
}

impl<T: Debug> Sender<T> {
    /// Send a value to the [`Receiver`]. This function does nothing if the
    /// [`Receiver`] has already closed up their connections.
    pub fn send(&self, item: T) {
        MainContext::default()
            .block_on(self.sender.send(item))
            .unwrap_or(());
    }
}

/// Receives values from the associated [`Sender`].
pub struct Receiver<T> {
    receiver: TokioReceiver<T>,
}

impl<T> Receiver<T> {
    /// Receive the value from the [`Sender`].
    pub fn recv(&mut self) -> T {
        let item = MainContext::default()
            .block_on(self.receiver.recv())
            .unwrap();
        item
    }
}

/// Return a tuple containing a [`Sender`] and [`Receiver`] that can be used to
/// send messages across a GTK application.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tokio_sender, tokio_receiver) = tokio_mpsc::channel(1);
    (
        Sender {
            sender: tokio_sender,
        },
        Receiver {
            receiver: tokio_receiver,
        },
    )
}
