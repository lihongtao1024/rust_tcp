use std::sync::Arc;
use bytes::Bytes;
use crate::Error;
use crate::socket;
use crate::listener;
use tokio::sync::mpsc::Sender;

pub enum Message {
    Bound(BoundMessage),
    Connected(ConnectedMessage),
    Received(ReceivedMessage),
    Error(ErrorMessage),
    Terminated(TerminatedMessage),
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

pub struct ReceivedMessage {
    pub socket: Arc<socket::Context>,
    pub bytes: Bytes,
}

pub struct ErrorMessage {
    pub socket: Arc<socket::Context>,
    pub err: Error,
}

pub struct TerminatedMessage {
    pub socket: Arc<socket::Context>,
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

    pub async fn send_received_message(sender: &Sender<Message>,
        socket: &Arc<socket::Context>, bytes: Bytes) {
        let _ = sender.send(Message::Received(ReceivedMessage {
            socket: socket.clone(),
            bytes,
        })).await;
    }

    pub async fn send_error_message(sender: &Sender<Message>, 
        socket: &Arc<socket::Context>, err: Error) {
        let _ = sender.send(Message::Error(ErrorMessage {
            socket: socket.clone(),
            err,
        })).await;
    }

    pub async fn send_terminated_message(sender: &Sender<Message>, 
        socket: &Arc<socket::Context>) {
        let _ = sender.send(Message::Terminated(TerminatedMessage {
            socket: socket.clone(),
        })).await;
    }

    pub async fn send_fatal_message(sender: &Sender<Message>, err: Error) {
        let _ = sender.send(Message::Fatal(FatalMessage {
            err,
        })).await;
    }
}
