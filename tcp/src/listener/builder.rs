use crate::Framer;
use crate::Message;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;

pub struct Builder {
    pub(crate) addr: SocketAddr,
    pub(crate) framer: Arc<dyn Framer>,
    pub(crate) message: MpscSender<Message>,
    pub(crate) shutdown: BroadcastReceiver<()>,
}

impl Builder {
    pub(crate) fn new(addr: SocketAddr, framer: Arc<dyn Framer>, 
        message: MpscSender<Message>, shutdown: BroadcastReceiver<()>) -> Self {
        Self { addr, framer, message, shutdown }
    }
}