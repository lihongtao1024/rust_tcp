use crate::Error;
use crate::Framer;
use crate::Message;
use crate::AsyncSocket;
use crate::Socket;
use crate::SocketBuilder;
use crate::SocketCreator;
use crate::ConnectionReader;
use crate::ConnectionWriter;
use async_trait::async_trait;
use tokio::select;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::net::SocketAddr;
use bytes::Bytes;

enum Event {
    Send(Bytes),
    Terminate,
}

#[derive(Debug)]
enum State {
    Connecting,
    Done,
    Shutdown,
    Disconnected,
}

pub struct DefaultSocket {
    state: AtomicU8,
    framer: Arc<dyn Framer>,
    local: Mutex<Option<SocketAddr>>,
    peer: Mutex<Option<SocketAddr>>,
    message: MpscSender<Message>,
    etx: UnboundedSender<Event>,
    erx: Mutex<Option<UnboundedReceiver<Event>>>,
    shutdown: BroadcastReceiver<()>,
    terminate: BroadcastSender<()>,
    tag: AtomicU64,
}

impl Debug for DefaultSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.load(Ordering::SeqCst);
        let state = match state {
            0 => State::Connecting,
            1 => State::Done,
            2 => State::Shutdown,
            3 => State::Disconnected,
            _ => panic!("system error"),
        };

        let local = self.local.lock().unwrap();
        let peer = self.peer.lock().unwrap();
        write!(
            f, 
            "Socket: {{ state:{:?}, local:{:?}, peer:{:?} }}", 
            state, 
            local, 
            peer
        )
    }
}

impl Display for DefaultSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Socket for DefaultSocket {
    fn send(self: Arc<Self>, bytes: Bytes) -> Result<(), Error> {
        if State::Done as u8 == self.state.load(Ordering::SeqCst) {
            self.etx.send(Event::Send(bytes))?;
            return Ok(());
        }

        Err(Error::Module("socket has not been established"))
    }
    
    fn disconnect(self: Arc<Self>) {
        let _ = self.state.compare_exchange(
            State::Done as u8,
            State::Shutdown as u8, 
            Ordering::SeqCst, 
            Ordering::SeqCst
        ).and_then(|_| {
            let _ = self.etx.send(Event::Terminate);
            Ok(())
        });
    }

    fn local_addr(self: Arc<Self>) -> Option<SocketAddr> {
        *self.local.lock().unwrap()
    }

    fn peer_addr(self: Arc<Self>) -> Option<SocketAddr> {
        *self.peer.lock().unwrap()
    }

    fn set_tag(self: Arc<Self>, tag: u64) {
        self.tag.store(tag, Ordering::SeqCst);
    }

    fn tag(self: Arc<Self>) -> u64 {
        self.tag.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl AsyncSocket for DefaultSocket {
    async fn accept(self: Arc<Self>, permit: OwnedSemaphorePermit, stream: TcpStream) {
        let (reader, writer) = stream.into_split();
        self.state.store(State::Done as u8, Ordering::SeqCst);

        let mut local = self.local.lock().unwrap();
        let _ = reader.local_addr().and_then(|addr| {
            *local = Some(addr);
            Ok(())
        });

        let mut peer = self.peer.lock().unwrap();
        let _ = reader.peer_addr().and_then(|addr| {
            *peer = Some(addr);
            Ok(())
        });

        let mut erx = self.erx.lock().unwrap();
        self.start(permit, reader, writer, erx.take().unwrap());
    }

    async fn connect(self: Arc<Self>, addr: SocketAddr, permit: OwnedSemaphorePermit) {
        let result = TcpStream::connect(addr).await;
        if let Err(err) = result {
            self.state.store(State::Disconnected as u8, Ordering::SeqCst);
            let _ = self.message.send(Message::ConnectFatal(self.clone(), err.into())).await;
            return;
        }

        self.clone().accept(permit, result.unwrap()).await;
        let _ = self.message.send(Message::ConnectDone(None, self.clone())).await;
    }
}

impl SocketCreator for DefaultSocket {
    fn new(builder: SocketBuilder) -> Arc<Self> {
        let (terminate, _) = broadcast::channel(1);
        let (etx, erx) = mpsc::unbounded_channel();
        Arc::new(
            Self {
                state: AtomicU8::new(State::Connecting as u8),
                framer: builder.framer,
                local: Mutex::new(None),
                peer: Mutex::new(None),
                message: builder.message,
                etx,
                erx: Mutex::new(Some(erx)),
                shutdown: builder.shutdown,
                terminate,
                tag: AtomicU64::new(0),
            }
        )
    }
}

impl DefaultSocket {
    fn start(self: &Arc<Self>, permit: OwnedSemaphorePermit, reader: OwnedReadHalf,
        writer: OwnedWriteHalf, receiver: UnboundedReceiver<Event>) -> JoinHandle<()> {
        let cloned = self.clone();
        let mut shutdown = self.shutdown.resubscribe();
        let mut terminate = self.terminate.subscribe();

        tokio::spawn(async move {
            select! {
                _ = cloned.read(reader) => {
                    drop(permit);
                    return;
                },
                _ = cloned.write(writer, receiver) => {
                    panic!("system error");
                },
                _ = terminate.recv() => {
                    cloned.state.store(State::Disconnected as u8, Ordering::SeqCst);
                    let _ = cloned.message.send(Message::ConnectTerminate(cloned.clone())).await;
                    drop(permit);
                    return;
                },
                _ = shutdown.recv() => {
                    cloned.state.store(State::Disconnected as u8, Ordering::SeqCst);
                    let _ = cloned.message.send(Message::ConnectTerminate(cloned.clone())).await;
                    drop(permit);
                    return;
                }
            }
        })
    }

    async fn read(self: &Arc<Self>, mut reader: OwnedReadHalf) {
        let mut connection = ConnectionReader::new(
            4096, 
            &mut reader, 
            &self.framer
        );

        loop {
            let result = connection.read_frame().await;
            match result {
                Ok(bytes) => {
                    match bytes {
                        Some(bytes) => {
                            let _ = self.message.send(Message::ReceiveDone(self.clone(), bytes)).await;
                        },
                        _ => {
                            self.state.store(State::Disconnected as u8, Ordering::SeqCst);
                            let _ = self.message.send(Message::ConnectTerminate(self.clone())).await;
                            return;
                        }
                    }
                },
                Err(err) => {
                    self.state.store(State::Disconnected as u8, Ordering::SeqCst);
                    let _ = self.message.send(Message::ConnectAbort(self.clone(), err)).await;
                    return;
                }
            }
        }
    }

    async fn write(self: &Arc<Self>, mut writer: OwnedWriteHalf, mut erx: UnboundedReceiver<Event>) {
        let mut connection = ConnectionWriter::new(&mut writer);

        while let Some(event) = erx.recv().await {
            match event {
                Event::Send(bytes) => {
                    let _ = connection.write_frame(bytes).await;
                },
                Event::Terminate => self.terminate().await,
            }
        }
    }

    async fn terminate(self: &Arc<Self>) {
        let _ = self.terminate.send(());
    }
}