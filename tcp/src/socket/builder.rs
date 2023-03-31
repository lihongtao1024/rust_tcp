use crate::Framer;
use crate::Message;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use std::sync::Arc;

pub struct Builder {
    pub(crate) framer: Arc<dyn Framer>,
    pub(crate) message: MpscSender<Message>,
    pub(crate) shutdown: BroadcastReceiver<()>,
}

impl Builder {
    pub(crate) fn new(framer: Arc<dyn Framer>, message: MpscSender<Message>,
        shutdown: BroadcastReceiver<()>) -> Self {
        Self { framer, message, shutdown }
    }
}