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

fn parse(data: &mut Cursor<&BytesMut>) -> Frame {
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


async fn bound_message(_listener: Arc<listener::Context>, _err: Error) {
}

async fn connected_message(_listener: Arc<listener::Context>, _socket: Arc<socket::Context>) {
}

async fn received_message(socket: Arc<socket::Context>, bytes: bytes::Bytes) {
    socket.send(bytes).await;
}

async fn error_message(_socket: Arc<socket::Context>, _err: Error) {
}

async fn terminated_message(_socket: Arc<socket::Context>) {
}

async fn fatal_message(_err: Error) {
}

struct Server<FA, FB, FC, FD, FE, FF>
where
    FA: Future<Output = ()> + Sync + Send + 'static,
    FB: Future<Output = ()> + Sync + Send + 'static,
    FC: Future<Output = ()> + Sync + Send + 'static,
    FD: Future<Output = ()> + Sync + Send + 'static,
    FE: Future<Output = ()> + Sync + Send + 'static,
    FF: Future<Output = ()> + Sync + Send + 'static {
    component: Component<FA, FB, FC, FD, FE, FF>,
    listener: Option<Listener>,
}

impl<FA, FB, FC, FD, FE, FF> Server<FA, FB, FC, FD, FE, FF>
where
    FA: Future<Output = ()> + Sync + Send + 'static,
    FB: Future<Output = ()> + Sync + Send + 'static,
    FC: Future<Output = ()> + Sync + Send + 'static,
    FD: Future<Output = ()> + Sync + Send + 'static,
    FE: Future<Output = ()> + Sync + Send + 'static,
    FF: Future<Output = ()> + Sync + Send + 'static {
    fn new(config: Config, parser: Parser, dispatcher: Dispatcher<FA, FB, FC, FD, FE, FF>) -> Self {
        Server {
            component: Component::new(config, parser, dispatcher),
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

    fn start(ip: String, port: u16, config: Config, parser: Parser, 
        dispatcher: Dispatcher<FA, FB, FC, FD, FE, FF>,
        mut shutdown: mpsc::Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut server = Server::new(config, parser, dispatcher);
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
        16,
        16,
        3000,
        4096,
    );
    let parser = Parser { parse };
    let dispatcher = Dispatcher {
        bound_message,
        connected_message,
        received_message,
        error_message,
        fatal_message,
        terminated_message,
    };

    let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
    let server = Server::start(
        "127.0.0.1".to_string(), 
        6668, 
        config,
        parser,
        dispatcher,
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
