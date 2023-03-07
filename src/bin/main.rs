use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::task::JoinHandle;
use tokio::select;
use tokio::sync::mpsc;
use tcp::Listener;
use tcp::Config;
use tcp::Error;
use tcp::Component;
use tcp::Dispatch;
use tcp::listener;
use tcp::socket;

struct MessageDispatch;
impl Dispatch for MessageDispatch {
    fn bound_message(&self, listener: Arc<listener::Context>, err: Error) {
        println!("bind listener success");
    }

    fn connected_message(&self, listener: Arc<listener::Context>, socket: Arc<socket::Context>) {
        println!("listener accept success");
    }

    fn error_message(&self, socket: Arc<socket::Context>, err: Error) {
        
    }

    fn fatal_message(&self, err: Error) {
        
    }
}

struct Server {
    component: Component,
    listener: Option<Listener>,
}

impl Server {
    fn new(config: Config, dispatch: Arc<dyn Dispatch + Send + Sync>) -> Server {
        Server {
            component: Component::new(config, dispatch),
            listener: None,
        }
    }

    async fn run(&mut self, shutdown: &mut mpsc::Receiver<()>) {
        loop {
            select! {
                _ = self.component.dispatch() => (),
                _ = shutdown.recv() => break
            }
        }
    }

    fn start(ip: String, port: u16, config: Config, dispatch: Arc<dyn Dispatch + Send + Sync>, 
        mut shutdown: mpsc::Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut server = Server::new(config, dispatch);
            server.listener = Some(server.component.listen(&ip, port).await);
            server.run(&mut shutdown).await;
            server.component.shutdown().await;
        })
    }
}

#[tokio::main]
async fn main() {
    let config = Config::new(
        4096, 
        4096, 
        3000,
        16,
        4096,
    );

    let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
    let server = Server::start(
        "127.0.0.1".to_string(), 
        6668, 
        config, 
        Arc::new(MessageDispatch{}), 
        shutdown_receiver
    );

    let _ = tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin()).lines();

        loop {
            let result = reader.next_line().await;
            if result.is_err() {
                break;
            }

            if let Some(line) = result.unwrap() {
                if line.eq(&"stop".to_string()) {
                    break;
                }
            }
        }

        shutdown_sender.send(()).await.unwrap();
        let _ = server.await;
    }).await;
}
