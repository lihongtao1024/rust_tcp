use bytes::Buf;
use bytes::BytesMut;
use std::mem;
use std::sync::Arc;
use std::future::Future;
use std::io::Cursor;
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
use tcp::Dispatcher;
use tcp::Parser;
use tcp::Frame;
use tcp::listener;
use tcp::socket;

struct Framer();
impl Parser for Framer {
    fn parse(&self, data: &mut Cursor<&BytesMut>) -> Frame {
        let len = data.get_ref().len();
        if len < mem::size_of::<u32>() {
            return Frame::Continue;
        }

        let size = data.get_u32_le();
        if len < size as usize {
            return Frame::Continue;
        }

        Frame::Success(size)
    }
}

async fn bound_message1(_listener: Arc<listener::Context>, _err: Error) {
    //println!("bind listener success");
}

async fn connected_message1(_listener: Arc<listener::Context>, _socket: Arc<socket::Context>) {
    //println!("listener accept success");
}

async fn received_message1(socket: Arc<socket::Context>, bytes: bytes::Bytes) {
    //println!("received message success, message size: {}", bytes.len());
    socket.send(bytes).await;
}

async fn error_message1(_socket: Arc<socket::Context>, _err: Error) {
    //println!("error message success");
}

async fn terminated_message1(_socket: Arc<socket::Context>) {
    //println!("terminate message success");
}

async fn fatal_message1(_err: Error) {
    //println!("fatal message success");
}

struct Server<F1, F2, F3, F4, F5, F6>
where
    F1: Future<Output = ()> + Sync + Send + 'static,
    F2: Future<Output = ()> + Sync + Send + 'static,
    F3: Future<Output = ()> + Sync + Send + 'static,
    F4: Future<Output = ()> + Sync + Send + 'static,
    F5: Future<Output = ()> + Sync + Send + 'static,
    F6: Future<Output = ()> + Sync + Send + 'static {
    component: Component<F1, F2, F3, F4, F5, F6>,
    listener: Option<Listener>,
}

impl<F1, F2, F3, F4, F5, F6> Server<F1, F2, F3, F4, F5, F6>
where
    F1: Future<Output = ()> + Sync + Send + 'static,
    F2: Future<Output = ()> + Sync + Send + 'static,
    F3: Future<Output = ()> + Sync + Send + 'static,
    F4: Future<Output = ()> + Sync + Send + 'static,
    F5: Future<Output = ()> + Sync + Send + 'static,
    F6: Future<Output = ()> + Sync + Send + 'static {
    fn new(config: Config<F1, F2, F3, F4, F5, F6>) -> Self {
        Server {
            component: Component::new(config),
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

    fn start(ip: String, port: u16, config: Config<F1, F2, F3, F4, F5, F6>, 
        mut shutdown: mpsc::Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut server = Server::new(config);
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
        16,
        16,
        3000,
        4096,
        Dispatcher {
            bound_message: bound_message1,
            connected_message: connected_message1,
            received_message: received_message1,
            error_message: error_message1,
            fatal_message: fatal_message1,
            terminated_message: terminated_message1,
        },
        Box::new(Framer()),
    );

    let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
    let server = Server::start(
        "127.0.0.1".to_string(), 
        6668, 
        config,
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
