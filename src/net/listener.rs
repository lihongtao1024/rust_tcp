use std::sync::Arc;
use std::future::Future;
use tokio::select;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
use crate::Error;
use crate::Config;
use crate::Message;
use crate::Socket;

pub struct Listener {
    internal: Arc<Context>,
}

pub struct Context {
    dispatch_sender: mpsc::Sender<Message>,
    conn_socket_permits: Arc<Semaphore>,
}

impl Listener {
    pub fn new<F1, F2, F3, F4, F5, F6>(addr: String, permit: OwnedSemaphorePermit, 
        conn_socket_permits: Arc<Semaphore>, config: Arc<Config<F1, F2, F3, F4, F5, F6>>, 
        dispatch_sender: mpsc::Sender<Message>, shutdown: broadcast::Receiver<()>) -> Listener 
    where
        F1: Future<Output = ()> + Sync + Send + 'static,
        F2: Future<Output = ()> + Sync + Send + 'static,
        F3: Future<Output = ()> + Sync + Send + 'static,
        F4: Future<Output = ()> + Sync + Send + 'static,
        F5: Future<Output = ()> + Sync + Send + 'static,
        F6: Future<Output = ()> + Sync + Send + 'static {
        let context = Arc::new(Context {
            dispatch_sender,
            conn_socket_permits,
        });

        let running_context = context.clone();
        let mut running_shutdown = shutdown.resubscribe();
        tokio::spawn(async move {
            select! {
                _ = running_context.run(addr, &config, shutdown) => (),
                _ = running_shutdown.recv() => {
                    drop(permit);
                },
            }
        });

        Self { internal: context}
    }

    pub fn equal(&self, context: &Arc<Context>) -> bool {
        Arc::ptr_eq(&self.internal, context)
    }
}

impl Context {
    async fn run<F1, F2, F3, F4, F5, F6>(self: &Arc<Self>, addr: String, 
        config: &Arc<Config<F1, F2, F3, F4, F5, F6>>, shutdown: broadcast::Receiver<()>)
    where
        F1: Future<Output = ()> + Sync + Send + 'static,
        F2: Future<Output = ()> + Sync + Send + 'static,
        F3: Future<Output = ()> + Sync + Send + 'static,
        F4: Future<Output = ()> + Sync + Send + 'static,
        F5: Future<Output = ()> + Sync + Send + 'static,
        F6: Future<Output = ()> + Sync + Send + 'static {
        let result = TcpListener::bind(addr).await;
        if let Err(err) = result {
            Message::send_fatal_message(
                &self.dispatch_sender, 
                Error::Io(err)
            ).await;
            return;
        }

        let listener = result.unwrap();
        loop {
            let permit = self.socket_permit().await;
            let result = listener.accept().await;
            if result.is_err() {
                continue;
            }

            let (socket, _) = result.unwrap();
            let socket = Socket::new(
                socket,
                permit,
                config.clone(),
                self.dispatch_sender.clone(), 
                shutdown.resubscribe()
            );
            Message::send_connected_message(
                &self.dispatch_sender, 
                self, 
                socket.context(),
            ).await;
        }
    }

    async fn socket_permit(&self) -> OwnedSemaphorePermit {
        self.conn_socket_permits.clone().acquire_owned().await.unwrap()
    }
}