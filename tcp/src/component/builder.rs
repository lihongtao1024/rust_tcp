use crate::Framer;
use crate::Dispatcher;
use crate::DefaultFramer;
use crate::ListenerCreator;
use crate::SocketCreator;
use crate::Component;
use crate::ComponentCreator;
use crate::Listener;
use crate::Socket;
use std::sync::Arc;

const DEFAULT_LISTENER_LIMIT: usize = 1;
const DEFAULT_SOCKET_LIMIT: usize = 256;
const DEFAULT_MESSAGE_LIMIT: usize = 512;

pub struct Builder {    
    pub(crate) listeners: usize,
    pub(crate) sockets: usize,
    pub(crate) messages: usize,
    pub(crate) framer: Arc<dyn Framer>,
    pub(crate) dispatcher: Option<&'static mut dyn Dispatcher>,
}

impl Builder {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            messages: DEFAULT_MESSAGE_LIMIT,
            listeners: DEFAULT_LISTENER_LIMIT,
            sockets: DEFAULT_SOCKET_LIMIT,
            framer: Arc::new(DefaultFramer::default()),
            dispatcher: None,
        }
    }

    #[allow(dead_code)]
    pub fn messages(mut self, n: usize) -> Self {
        self.messages = n;
        self
    }

    #[allow(dead_code)]
    pub fn listener(mut self, n: usize) -> Self {
        self.listeners = n;
        self
    }

    #[allow(dead_code)]
    pub fn sockets(mut self, n: usize) -> Self {
        self.sockets = n;
        self
    }

    #[allow(dead_code)]
    pub fn framer(mut self, framer: Arc<dyn Framer>) -> Self {
        self.framer = framer;
        self
    }

    #[allow(dead_code)]
    pub fn dispatcher(mut self, dispatcher: &'static mut dyn Dispatcher) -> Self {
        self.dispatcher = Some(dispatcher);
        self
    }

    #[allow(dead_code)]
    pub fn build<T, L, S> (self) -> impl Component<L, S>
    where
        T: ComponentCreator<L, S> + Component<L, S>,
        L: ListenerCreator + Listener,
        S: SocketCreator + Socket,
    {
        T::new(self)
    }
}