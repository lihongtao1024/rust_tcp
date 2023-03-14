use crate::Error;
use crate::Parser;
use crate::Message;
use crate::Manager;
use crate::ConnectionReader;
use crate::ConnectionWriter;
use bytes::Bytes;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::select;
use tokio::io;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Weak;

pub enum State {
    Connecting,
    Done,
    Disconnected,
}

pub struct Socket {
    state: RwLock<State>,
    owner: Weak<Manager>,
    parser: Arc<Parser>,
    reader: RwLock<Option<ReadHalf<TcpStream>>>,
    writer: RwLock<Option<WriteHalf<TcpStream>>>,    
    dispatcher: mpsc::UnboundedSender<Message>,
    shutdown: broadcast::Receiver<()>,
}

impl Socket {
    pub fn new(manager: Weak<Manager>, parser: Arc<Parser>, 
        dispatcher: mpsc::UnboundedSender<Message>, 
        shutdown: broadcast::Receiver<()>) -> Arc<Socket> {
        Arc::new(
            Socket {
                parser,
                dispatcher,
                shutdown,
                owner: manager,
                reader: RwLock::new(None),
                writer: RwLock::new(None),
                state: RwLock::new(State::Connecting),
            }
        )
    }

    pub fn send_to(self: &Arc<Socket>, bytes: Bytes) -> bool {
        let manager = self.owner.upgrade();
        if manager.is_none() {
            return false;
        }

        let manager = manager.unwrap();
        if let Err(_) = manager.send(self, bytes) {
            return false;
        }

        return true;
    }

    pub async fn accept(self: &Arc<Socket>, permit: OwnedSemaphorePermit, socket: TcpStream) {
        let (read, write) = io::split(socket);

        let mut state = self.state.write().await;
        *state = State::Done;

        let mut reader = self.reader.write().await;
        *reader = Some(read);

        let mut writer = self.writer.write().await;
        *writer = Some(write);

        self.start(permit);
    }

    pub async fn connect(self: &Arc<Socket>, permit: OwnedSemaphorePermit, addr: SocketAddr) {
        let result = TcpStream::connect(addr).await;
        if let Err(err) = result {
            let mut state = self.state.write().await;
            *state = State::Disconnected;

            let message = Message::ConnectFatal(self.clone(), Error::Io(err));
            let _ = self.dispatcher.send(message);
            return;
        }

        self.accept(permit, result.unwrap()).await;
    }

    pub async fn send(self: &Arc<Socket>, bytes: Bytes) {
        let state = self.state.read().await;
        match &*state {
            State::Done => (),
            _ => return,
        }

        let mut writer = self.writer.write().await;
        match &mut *writer {
            Some(writer) => {
                let mut connection = ConnectionWriter::new(writer);
                let _ = connection.write_frame(bytes).await;
            },
            _ => return,
        }
    }

    fn start(self: &Arc<Socket>, permit: OwnedSemaphorePermit) {
        let socket = self.clone();
        let mut shutdown = self.shutdown.resubscribe();

        tokio::spawn(async move {
            select! {
                _ = socket.read() => (),
                _ = shutdown.recv() => {
                    drop(permit);
                    return;
                }
            }
        });
    }

    async fn read(self: &Arc<Socket>) {
        let mut reader = self.reader.write().await;

        match &mut *reader {
            Some(reader) => {
                let mut connection = ConnectionReader::new(
                    4096, 
                    &mut *reader, 
                    &self.parser,
                );

                loop {
                    let result = connection.read_frame().await;
                    if let Err(err) = result {
                        let mut state = self.state.write().await;
                        *state = State::Disconnected;
    
                        let message = Message::ConnectAbort(self.clone(), err);
                        let _ = self.dispatcher.send(message);
                        return;
                    }
    
                    let result = result.ok().unwrap();
                    if let Some(bytes) = result {
                        let message = Message::ReceiveDone(self.clone(), bytes);
                        let _ = self.dispatcher.send(message);
                    } else {
                        let mut state = self.state.write().await;
                        *state = State::Disconnected;
    
                        let message = Message::ConnectTerminate(self.clone());
                        let _ = self.dispatcher.send(message);
                        return;
                    }
                }
            },
            _ => assert!(false),
        }
    }
}