use crate::Config;
use crate::Message;
use crate::Result;
use crate::Parser;
use crate::Dispatcher;
use crate::Event;
use crate::Listener;
use crate::Socket;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::runtime::Builder;
use tokio::select;
use bytes::Bytes;
use std::net::SocketAddr;
use std::thread;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Manager {
    config: Config,
    dispatcher: Dispatcher,
    parser: Arc<Parser>,
    listener_semaphore: Arc<Semaphore>,
    connection_semaphore: Arc<Semaphore>,
    event_tx: mpsc::UnboundedSender<Event>,
    dispatch_tx: mpsc::UnboundedSender<Message>,
    dispatch_rx: RwLock<mpsc::UnboundedReceiver<Message>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Manager {
    pub fn new(config: Config, dispatcher: Dispatcher, parser: Parser, 
        shutdown: broadcast::Receiver<()>) -> Arc<Manager> {
        let listener_semaphore = Arc::new(Semaphore::new(
            config.bind_listener_limit)
        );
        let connection_semaphore = Arc::new(
            Semaphore::new(config.conn_socket_limit)
        );
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (dispatch_tx, dispatch_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, _) = broadcast::channel(1);

        let manager = Arc::new(Manager {
            config,
            dispatcher,
            listener_semaphore,
            connection_semaphore,
            event_tx,
            dispatch_tx,            
            shutdown_tx,
            dispatch_rx: RwLock::new(dispatch_rx),
            parser: Arc::new(parser),
        });

        manager.start(event_rx, shutdown);
        return manager;
    }

    pub fn listen(self: &Arc<Manager>, addr: SocketAddr) -> Result<Arc<Listener>> {
        let listener = Listener::new(
            self,
            self.parser.clone(),
            self.dispatch_tx.clone(),
            self.shutdown_tx.subscribe(),
        );

        let event = Event::Listen(addr, listener.clone());
        self.event_tx.send(event)?;
        Ok::<Arc<Listener>, _>(listener)
    }

    pub fn connect(self: &Arc<Manager>, addr: SocketAddr) -> Result<Arc<Socket>> {
        let socket = Socket::new(
            Arc::downgrade(self), 
            self.parser.clone(), 
            self.dispatch_tx.clone(),
            self.shutdown_tx.subscribe(),
        );

        let event = Event::Connect(addr, socket.clone());
        self.event_tx.send(event)?;
        Ok::<Arc<Socket>, _>(socket)
    }

    pub fn send(self: &Arc<Manager>, socket: &Arc<Socket>, bytes: Bytes) -> Result<()> {
        let event = Event::Send(socket.clone(), bytes);
        self.event_tx.send(event)?;
        Ok::<(), _>(())
    }

    pub fn dispatch(self: &Arc<Manager>, count: u8) {
        let mut rx = self.dispatch_rx.write().unwrap();
        for _ in 0..count {
            match rx.try_recv() {
                Ok(message) => {
                    match message {
                        Message::ListenFatal(listener, err) => {
                            (self.dispatcher.listen_fatal)(listener.clone(), err);
                        },
                        Message::ConnectDone(listener, socket) => {
                            (self.dispatcher.connect_done)(listener, socket);
                        },
                        Message::ConnectAbort(socket, err) => {
                            (self.dispatcher.connect_abort)(socket, err);
                        },
                        Message::ConnectFatal(socket, err) => {
                            (self.dispatcher.connect_fatal)(socket, err);
                        },
                        Message::ConnectTerminate(socket) => {
                            (self.dispatcher.connect_terminate)(socket);
                        },
                        Message::ReceiveDone(socket, bytes) => {
                            (self.dispatcher.receive_done)(socket, bytes);
                        }
                     }
                },
                _ => return,
            }
        }
    }
 
    fn start(self: &Arc<Manager>, event_rx: mpsc::UnboundedReceiver<Event>,
        shutdown: broadcast::Receiver<()>) {
        let manager = self.clone();
        thread::spawn(move || {
            Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(manager.run(event_rx, shutdown));
        });
    }

    async fn run(self: &Arc<Manager>, event_rx: mpsc::UnboundedReceiver<Event>,
        mut shutdown: broadcast::Receiver<()>) {
        select! {
            _ = self.run_event(event_rx) => (),
            _ = shutdown.recv() => {
                let _ = self.shutdown_tx.send(());
                self.wait().await;
                return;
            }
        }
    }

    async fn wait(self: &Arc<Manager>) {
        while self.listener_semaphore.available_permits() != self.config.bind_listener_limit {
            tokio::task::yield_now().await;
        }

        while self.connection_semaphore.available_permits() != self.config.conn_socket_limit {
            tokio::task::yield_now().await;
        }
    }

    async fn run_event(self: &Arc<Manager>, mut event_rx: mpsc::UnboundedReceiver<Event>) {
        while let Some(evt) = event_rx.recv().await {
            match &evt {
                Event::Listen(addr, listener) => {
                    let permit = self.listener_semaphore
                        .clone()
                        .acquire_owned()
                        .await
                        .unwrap();
                    listener.bind(permit, *addr, self.connection_semaphore.clone()).await;
                },
                Event::Connect(addr, socket) => {
                    let permit = self.connection_semaphore
                        .clone()
                        .acquire_owned()
                        .await
                        .unwrap();
                    socket.connect(permit, *addr).await;
                },
                Event::Send(socket, bytes) => {
                    socket.send(bytes.clone()).await;
                }
            }
        }
    }
}