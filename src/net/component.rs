use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::broadcast;
use tokio::task;
use crate::Config;
use crate::Dispatch;
use crate::Result;
use crate::Message;
use crate::Listener;

pub struct Component {
    internal: Arc<Context>,
    messasge_dispatch: Arc<dyn Dispatch + Send + Sync>,
    message_sender: mpsc::Sender<Message>,
    message_receiver: mpsc::Receiver<Message>,
    shutdown_receiver: broadcast::Receiver<()>,
    shutdown_sender: broadcast::Sender<()>,    
}

pub struct Context {
    config: Config,
    limit_listeners: Arc<Semaphore>,
    limit_sockets: Arc<Semaphore>,    
}

impl Component {
    pub fn new(config: Config, dispatch: Arc<dyn Dispatch + Send + Sync>) -> Self {
        let listener_limit = config.listener_limit;
        let limit_listeners = Arc::new(Semaphore::new(listener_limit));

        let socket_limit = config.socket_limit;
        let limit_sockets = Arc::new(Semaphore::new(socket_limit));

        let (shutdown_sender, shutdown_receiver) = broadcast::channel(
            1
        );
        let (message_sender, message_receiver) = mpsc::channel(
            config.message_limit
        );

        let context = Arc::new(
            Context { 
                config,
                limit_listeners,
                limit_sockets,                
            }
        );

        Self {
            internal: context,
            messasge_dispatch: dispatch,
            message_sender,
            message_receiver,
            shutdown_receiver,         
            shutdown_sender,
        }
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown_sender.send(());

        while self.internal.limit_listeners.available_permits() != self.internal.config.listener_limit {
            task::yield_now().await;
        }

        while self.internal.limit_sockets.available_permits() != self.internal.config.socket_limit {
            task::yield_now().await;
        }
    }

    pub async fn listen(&self, ip: &str, port: u16) -> Listener {
        let addr = format!("{}:{}", ip, port);
        Listener::new(
            self.internal.listener_permit().await,
            addr, 
            self.message_sender.clone(), 
            self.shutdown_receiver.resubscribe()
        )
    }

    pub async fn dispatch(&mut self) {
        if let Some(message) = self.message_receiver.recv().await {
            match message {
                Message::Bound(message) => {
                    self.messasge_dispatch.bound_message(message.listener, message.err);
                },
                Message::Connected(message) => {
                    self.messasge_dispatch.connected_message(message.listener, message.socket);
                },
                Message::Error(message) => {
                    self.messasge_dispatch.error_message(message.socket, message.err);
                }
                Message::Fatal(message) => {
                    self.messasge_dispatch.fatal_message(message.err);
                }
            }
        }
    }

}

impl Context {
    async fn listener_permit(&self) -> OwnedSemaphorePermit {
        self.limit_listeners.clone().acquire_owned().await.unwrap()
    }
}