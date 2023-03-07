use std::sync::Arc;
use tokio::select;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::mpsc;
use crate::Error;
use crate::Message;
use crate::Socket;

pub struct Listener {
    internal: Arc<Context>,
}

pub struct Context {
    message: mpsc::Sender<Message>,
}

impl Listener {
    pub fn new(permit: OwnedSemaphorePermit, addr: String, message: mpsc::Sender<Message>, 
        shutdown: broadcast::Receiver<()>) -> Listener {
        let context = Arc::new(Context {
            message,
        });

        let running_context = context.clone();
        let mut running_shutdown = shutdown.resubscribe();
        tokio::spawn(async move {
            println!("run listener");
            select! {
                _ = running_context.run(addr, shutdown) => (),
                _ = running_shutdown.recv() => {
                    drop(permit);
                    println!("shutdown listener");
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
    async fn run(self: &Arc<Self>, addr: String, shutdown: broadcast::Receiver<()>) {
        let result = TcpListener::bind(addr).await;
        if let Err(err) = result {
            Message::send_fatal_message(
                &self.message, 
                Error::Io(err)
            ).await;
            return;
        }

        let listener = result.unwrap();
        loop {
            let result = listener.accept().await;
            if result.is_err() {
                continue;
            }

            let (socket, addr) = result.unwrap();
            let socket = Socket::new(
                socket, 
                addr, 
                self.message.clone(), 
                shutdown.resubscribe()
            );
            Message::send_connected_message(
                &self.message, 
                self, 
                socket.context(),
            ).await;
        }
    }
}