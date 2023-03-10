use std::sync::Arc;
use std::future::Future;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::broadcast;
use tokio::task;
use crate::Config;
use crate::Dispatcher;
use crate::Message;
use crate::Listener;
use crate::Parser;

pub struct Component<FA, FB, FC, FD, FE, FF>
where
    FA: Future<Output = ()> + Sync + Send + 'static,
    FB: Future<Output = ()> + Sync + Send + 'static,
    FC: Future<Output = ()> + Sync + Send + 'static,
    FD: Future<Output = ()> + Sync + Send + 'static,
    FE: Future<Output = ()> + Sync + Send + 'static,
    FF: Future<Output = ()> + Sync + Send + 'static {
    config: Arc<Config>,
    parser: Arc<Parser>,
    bind_listener_permits: Arc<Semaphore>,
    conn_socket_permits: Arc<Semaphore>,    
    dispatcher: Dispatcher<FA, FB, FC, FD, FE,FF>,
    dispatch_sender: mpsc::Sender<Message>,
    dispatch_receiver: mpsc::Receiver<Message>,
    shutdown_receiver: broadcast::Receiver<()>,
    shutdown_sender: broadcast::Sender<()>,
}

impl<FA, FB, FC, FD, FE, FF> Component<FA, FB, FC, FD, FE, FF>
where
    FA: Future<Output = ()> + Sync + Send + 'static,
    FB: Future<Output = ()> + Sync + Send + 'static,
    FC: Future<Output = ()> + Sync + Send + 'static,
    FD: Future<Output = ()> + Sync + Send + 'static,
    FE: Future<Output = ()> + Sync + Send + 'static,
    FF: Future<Output = ()> + Sync + Send + 'static {
    pub fn new(config: Config, parser: Parser, dispatcher: Dispatcher<FA, FB, FC, FD, FE, FF>) -> Self {
        let bind_listener_limit = config.bind_listener_limit;
        let bind_listener_permits = Arc::new(
            Semaphore::new(bind_listener_limit)
        );
        let conn_socket_limit = config.conn_socket_limit;
        let conn_socket_permits = Arc::new(
            Semaphore::new(conn_socket_limit)
        );
        let (shutdown_sender, shutdown_receiver) = broadcast::channel(
            1
        );
        let (dispatch_sender, dispatch_receiver) = mpsc::channel(
            config.dispatch_queue_limit
        );

        Self {
            config: Arc::new(config),
            parser: Arc::new(parser),
            bind_listener_permits,
            conn_socket_permits,
            dispatcher,
            dispatch_sender,
            dispatch_receiver,
            shutdown_receiver,
            shutdown_sender,
        }
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown_sender.send(());

        while self.conn_socket_permits.available_permits() != self.config.conn_socket_limit {
            task::yield_now().await;
        }

        while self.bind_listener_permits.available_permits() != self.config.bind_listener_limit {
            task::yield_now().await;
        }
    }

    pub async fn listen(&self, ip: &str, port: u16) -> Listener {
        Listener::new(
            format!("{}:{}", ip, port),
            self.listener_permit().await,
            self.conn_socket_permits.clone(),            
            self.config.clone(),
            self.parser.clone(),
            self.dispatch_sender.clone(), 
            self.shutdown_receiver.resubscribe(),
        )
    }

    pub async fn dispatch(&mut self) {
        if let Some(message) = self.dispatch_receiver.recv().await {
            let dispatcher = &self.dispatcher;
            match message {
                Message::Bound(message) => {
                    (dispatcher.bound_message)(message.listener, message.err).await;
                },
                Message::Connected(message) => {
                    (dispatcher.connected_message)(message.listener, message.socket).await;
                },
                Message::Received(message) => {
                    (dispatcher.received_message)(message.socket, message.bytes).await;
                },
                Message::Error(message) => {
                    (dispatcher.error_message)(message.socket, message.err).await;
                },
                Message::Terminated(message) => {
                    (dispatcher.terminated_message)(message.socket).await;
                }
                Message::Fatal(message) => {
                    (dispatcher.fatal_message)(message.err).await;
                }
            }
        }
    }

    async fn listener_permit(&self) -> OwnedSemaphorePermit {
        self.bind_listener_permits.clone().acquire_owned().await.unwrap()
    }
}