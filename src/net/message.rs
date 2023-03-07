use std::sync::Arc;
use crate::Error;
use crate::socket;
use crate::listener;
use tokio::sync::mpsc::Sender;

pub enum Message {
    Bound(BoundMessage),
    Connected(ConnectedMessage),
    Error(ErrorMessage),
    Fatal(FatalMessage),
}

pub struct BoundMessage {
    pub listener: Arc<listener::Context>,
    pub err: Error,
}

pub struct ConnectedMessage {
    pub listener: Arc<listener::Context>,
    pub socket: Arc<socket::Context>,
}

pub struct ErrorMessage {
    pub socket: Arc<socket::Context>,
    pub err: Error,
}

pub struct FatalMessage {
    pub err: Error,
}

impl Message {
    pub async fn send_bound_message(sender: &Sender<Message>, 
        listener: &Arc<listener::Context>, err: Error) {
        let _ = sender.send(Message::Bound(BoundMessage {
            listener: listener.clone(),
            err
        })).await;
    }

    pub async fn send_connected_message(sender: &Sender<Message>, 
        listener: &Arc<listener::Context>, socket: &Arc<socket::Context>) {
        let _ = sender.send(Message::Connected(ConnectedMessage {
            listener: listener.clone(),
            socket: socket.clone(),
        })).await;
    }

    pub async fn send_error_message(sender: &Sender<Message>, 
        socket: &Arc<socket::Context>, err: Error) {
            let _ = sender.send(Message::Error(ErrorMessage {
                socket: socket.clone(),
                err,
            })).await;
    }

    pub async fn send_fatal_message(sender: &Sender<Message>, err: Error) {
            let _ = sender.send(Message::Fatal(FatalMessage {
                err,
            })).await;
    }
}
