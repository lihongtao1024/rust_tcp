use crate::Framer;
use crate::Message;
use crate::AsyncListener;
use crate::Listener;
use crate::ListenerBuilder;
use crate::ListenerCreator;
use crate::Socket;
use crate::SocketBuilder;
use async_trait::async_trait;
use tokio::select;
use tokio::task::JoinHandle;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::Semaphore;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use std::sync::Arc;
use std::fmt::Debug;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

#[derive(Debug)]
enum State {
    Binding,
    Done,
    Fatal,
    Unbound,
}

pub struct DefaultListener {
    addr: SocketAddr,
    state: AtomicU8,
    framer: Arc<dyn Framer>,
    message: MpscSender<Message>,
    shutdown: BroadcastReceiver<()>,
    close: BroadcastSender<()>,
}

impl Debug for DefaultListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.load(Ordering::SeqCst);
        let state = match state {
            0 => State::Binding,
            1 => State::Done,
            2 => State::Fatal,
            3 => State::Unbound,
            _ => panic!("system error"),
        };
        write!(f, "Listener: {{ state:{:?}, addr:{:?} }}", state, self.addr)
    }
}

impl Display for DefaultListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Listener for DefaultListener {
    fn close(self: Arc<Self>) {
        let _ = self.close.send(());
    }
}

#[async_trait]
impl AsyncListener for DefaultListener {
    async fn bind(self: Arc<Self>, permit: OwnedSemaphorePermit, semaphore: Arc<Semaphore>,
        allocator: fn(SocketBuilder) -> Arc<dyn Socket>) {
        let result = TcpListener::bind(self.addr).await;
        if let Err(err) = result {
            self.state.store(State::Fatal as u8, Ordering::SeqCst);
            let _ = self.message.send(Message::ListenFatal(self.clone(), err.into())).await;
            return;
        }

        self.state.store(State::Done as u8, Ordering::SeqCst);
        self.start(permit, semaphore, result.unwrap(), allocator);
    }
}

impl ListenerCreator for DefaultListener {
    fn new(builder: ListenerBuilder) -> Arc<Self> {
        let (close, _) = broadcast::channel(1);
        Arc::new(Self {
            addr: builder.addr,
            state: AtomicU8::new(State::Binding as u8),
            framer: builder.framer,
            message: builder.message,
            shutdown: builder.shutdown,
            close,
        })
    }
}

impl DefaultListener {
    fn start(self: &Arc<Self>, permit: OwnedSemaphorePermit, semaphore: Arc<Semaphore>, 
        tcp: TcpListener, allocator: fn(SocketBuilder) -> Arc<dyn Socket>) -> JoinHandle<()> {
        let cloned = self.clone();
        let mut shutdown = self.shutdown.resubscribe();
        let mut close = self.close.subscribe();

        tokio::spawn(async move {
            select! {
                _ = cloned.run(semaphore, tcp, allocator) => {
                    panic!("system error");
                },
                _ = close.recv() => {
                    cloned.state.store(State::Unbound as u8, Ordering::SeqCst);
                    drop(permit);
                    return;
                },
                _ = shutdown.recv() => {
                    cloned.state.store(State::Unbound as u8, Ordering::SeqCst);
                    drop(permit);
                    return;
                }
            }
        })
    }

    async fn run(self: &Arc<Self>, semaphore: Arc<Semaphore>, tcp: TcpListener, 
        allocator: fn(SocketBuilder) -> Arc<dyn Socket>) {
        loop {
                let result = tcp.accept().await;
                let (stream, _) = result.unwrap();
                let permit = semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap();

                let builder = SocketBuilder::new(
                    self.framer.clone(),
                    self.message.clone(),
                    self.shutdown.resubscribe(),
                );
                let socket = allocator(builder);
                socket.clone().accept(permit, stream).await;
                let _ = self.message.send(Message::ConnectDone(Some(self.clone()), socket)).await;
            }
    }
}