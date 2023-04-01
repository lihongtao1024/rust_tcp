use crate::Framer;
use crate::Message;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use std::sync::Arc;

const DEFAULT_EVENTS: usize = 16;

pub struct Builder {
    pub(crate) events: usize,
    pub(crate) framer: Arc<dyn Framer>,
    pub(crate) message: MpscSender<Message>,
    pub(crate) shutdown: BroadcastReceiver<()>,
}

impl Builder {
    pub(crate) fn new(framer: Arc<dyn Framer>, message: MpscSender<Message>,
        shutdown: BroadcastReceiver<()>) -> Self {
        Self { events: DEFAULT_EVENTS, framer, message, shutdown }
    }

    #[allow(dead_code)]
    pub(crate) fn events(mut self, events: usize) -> Self {
        self.events = events;
        self
    }
}