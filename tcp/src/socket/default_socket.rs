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
use tokio::sync::mpsc::Receiver as MpscReceiver;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::task::JoinHandle;
use std::cell::SyncUnsafeCell;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
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
    local: SyncUnsafeCell<Option<SocketAddr>>,
    peer: SyncUnsafeCell<Option<SocketAddr>>,
    message: MpscSender<Message>,
    etx: MpscSender<Event>,
    erx: SyncUnsafeCell<Option<MpscReceiver<Event>>>,
    shutdown: BroadcastReceiver<()>,
    terminate: BroadcastSender<()>,
    tag: SyncUnsafeCell<Option<usize>>,
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

        let local = unsafe { *self.local.get() };
        let peer = unsafe { *self.peer.get() };
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
            match self.etx.try_send(Event::Send(bytes.clone())) {
                Ok(_) => return Ok(()),
                Err(TrySendError::Closed(_)) => (),
                Err(TrySendError::Full(_)) => {
                    self.etx.blocking_send(Event::Send(bytes))?;
                    return Ok(());
                },
            }
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
        unsafe { *self.local.get() }
    }

    fn peer_addr(self: Arc<Self>) -> Option<SocketAddr> {
        unsafe { *self.peer.get() }
    }

    fn set_tag(self: Arc<Self>, tag: usize) {
        unsafe { *self.tag.get() = Some(tag) };
    }

    fn tag(self: Arc<Self>) -> Option<usize> {
        unsafe { *self.tag.get() }
    }
}

#[async_trait]
impl AsyncSocket for DefaultSocket {
    async fn accept(self: Arc<Self>, permit: OwnedSemaphorePermit, stream: TcpStream) {
        let (reader, writer) = stream.into_split();
        self.state.store(State::Done as u8, Ordering::SeqCst);

        let _ = reader.local_addr().and_then(|addr| {
            unsafe { *self.local.get() = Some(addr); }
            Ok(())
        });

        let _ = reader.peer_addr().and_then(|addr| {
            unsafe { *self.peer.get() = Some(addr) };
            Ok(())
        });

        self.start(permit, reader, writer, unsafe { (*self.erx.get()).take().unwrap() });
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
        let (etx, erx) = mpsc::channel(builder.events);
        Arc::new(
            Self {
                state: AtomicU8::new(State::Connecting as u8),
                framer: builder.framer,
                local: SyncUnsafeCell::new(None),
                peer: SyncUnsafeCell::new(None),
                message: builder.message,
                etx,
                erx: SyncUnsafeCell::new(Some(erx)),
                shutdown: builder.shutdown,
                terminate,
                tag: SyncUnsafeCell::new(None),
            }
        )
    }
}

impl DefaultSocket {
    fn start(self: &Arc<Self>, permit: OwnedSemaphorePermit, reader: OwnedReadHalf,
        writer: OwnedWriteHalf, receiver: MpscReceiver<Event>) -> JoinHandle<()> {
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
            match connection.read_frame().await {
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

    async fn write(self: &Arc<Self>, mut writer: OwnedWriteHalf, mut erx: MpscReceiver<Event>) {
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