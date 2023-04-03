use crate::Framer;
use crate::Dispatcher;
use crate::DefaultFramer;
use crate::ListenerCreator;
use crate::SocketCreator;
use crate::Component;
use crate::ComponentCreator;
use crate::ServerComponent;
use crate::Listener;
use crate::Socket;
use std::sync::Arc;

const DEFAULT_LISTENER_LIMIT: usize = 0;
const DEFAULT_SOCKET_LIMIT: usize = 256;
const DEFAULT_MESSAGE_LIMIT: usize = 512;
const DEFAULT_EVENT_LIMIT: usize = 128;
const DEFAULT_DISPATCH_LIMIT: usize = 8;
const DEFAULT_SOCKET_EVENTS: usize = 64;

pub struct Builder {    
    pub(crate) listeners: usize,
    pub(crate) sockets: usize,
    pub(crate) messages: usize,
    pub(crate) events: usize,
    pub(crate) dispatchs: usize,
    pub(crate) socket_events: usize,
    pub(crate) framer: Arc<dyn Framer>,
    pub(crate) dispatcher: Option<&'static mut dyn Dispatcher>,
}

impl Builder {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            listeners: DEFAULT_LISTENER_LIMIT,
            sockets: DEFAULT_SOCKET_LIMIT,
            messages: DEFAULT_MESSAGE_LIMIT,
            events: DEFAULT_EVENT_LIMIT,
            dispatchs: DEFAULT_DISPATCH_LIMIT,
            socket_events: DEFAULT_SOCKET_EVENTS,
            framer: Arc::new(DefaultFramer::default()),
            dispatcher: None,
        }
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
    pub fn messages(mut self, n: usize) -> Self {
        self.messages = n;
        self
    }

    #[allow(dead_code)]
    pub fn events(mut self, n: usize) -> Self {
        self.events = n;
        self
    }

    #[allow(dead_code)]
    pub fn dispatchs(mut self, n: usize) -> Self {
        self.dispatchs = n;
        self
    }

    #[allow(dead_code)]
    pub fn socket_events(mut self, n: usize) -> Self {
        self.socket_events = n;
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
    pub fn build<T, S> (self) -> impl Component<S>
    where
        T: ComponentCreator<S> + Component<S>,
        S: SocketCreator + Socket,
    {
        T::new(self)
    }

    #[allow(dead_code)]
    pub fn build_server<T, L, S> (self) -> impl ServerComponent<L, S>
    where
        T: ComponentCreator<S> + ServerComponent<L, S>,
        L: ListenerCreator + Listener,
        S: SocketCreator + Socket,
    {
        T::new(self)
    }
}