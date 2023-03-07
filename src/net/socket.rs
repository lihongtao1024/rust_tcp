use std::sync::Arc;
use std::net::SocketAddr;
use tokio::select;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::broadcast::Receiver;
use crate::Message;

pub struct Socket {
    internal: Arc<Context>,
    addr: SocketAddr,
}

pub struct Context {
    socket: TcpStream,
    message: mpsc::Sender<Message>,
}

impl Socket {
    pub fn new(socket: TcpStream, addr: SocketAddr, message: Sender<Message>, 
        mut shutdown: Receiver<()>) -> Self {
        let context = Arc::new(Context {
            socket,
            message,
        });

        let running_context = context.clone();
        tokio::spawn(async move {
            println!("run socket");
            select! {
                _ = running_context.run() => (),
                _ = shutdown.recv() => {
                    println!("shutdown socket");
                }
            }
        });

        Self { internal: context, addr }
    }

    pub fn context(&self) -> &Arc<Context> {
        &self.internal
    }

    pub fn equal(&self, context: &Arc<Context>) -> bool {
        Arc::ptr_eq(&self.internal, context)
    }
}

impl Context {
    async fn run(&self) {
        let never = std::future::pending::<()>();
        never.await;
    }
}