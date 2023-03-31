use crate::Error;
use crate::Framer;
use crate::Message;
use crate::Dispatcher;
use crate::Listener;
use crate::ListenerBuilder;
use crate::ListenerCreator;
use crate::Socket;
use crate::SocketBuilder;
use crate::SocketCreator;
use crate::Component;
use crate::ComponentBuilder;
use crate::ComponentCreator;
use std::thread;
use std::sync::Arc;
use std::net::SocketAddr;
use std::fmt::Debug;
use std::fmt::Display;
use std::thread::JoinHandle;
use tokio::select;
use tokio::task;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::mpsc::Receiver as MpscReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::runtime::Builder as RuntimeBuilder;

const DEFAULT_DISPATCH_COUNT: usize = 16;

enum Event {
    Listen(Arc<dyn Listener>),
    Connect(SocketAddr, Arc<dyn Socket>),
}

struct ThreadContext {
    listeners: Arc<Semaphore>,
    sockets: Arc<Semaphore>,
}

pub struct DefaultComponent {
    framer: Arc<dyn Framer>,
    mtx: MpscSender<Message>,
    mrx: MpscReceiver<Message>,
    etx: UnboundedSender<Event>,
    stx: BroadcastSender<()>,
    dispatcher: Option<&'static mut dyn Dispatcher>,
    context: Arc<ThreadContext>,
    wait: JoinHandle<()>,
}

impl ThreadContext {
    fn new(listeners: usize, socket: usize) -> Arc<Self> {
        Arc::new(
            Self {
                listeners: Arc::new(Semaphore::new(listeners)),
                sockets: Arc::new(Semaphore::new(socket)),
            }
        )
    }

    fn start(self: &Arc<Self>, erx: UnboundedReceiver<Event>, 
        srx: BroadcastReceiver<()>, listeners: usize, sockets: usize, 
        allocator: fn(SocketBuilder) -> Arc<dyn Socket>) -> JoinHandle<()> {
        let cloned = self.clone();

        thread::spawn(move || {
            RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(cloned.run(erx, srx, listeners, sockets, allocator));
        })
    }

    async fn run(self: &Arc<Self>, erx: UnboundedReceiver<Event>, mut srx: BroadcastReceiver<()>,
        listeners: usize, sockets: usize, allocator: fn(SocketBuilder) -> Arc<dyn Socket>) {
        select! {
            _ = self.handle(erx, allocator) => {
                panic!("system error");
            },
            _ = srx.recv() => {
                self.wait(listeners, sockets).await;
            }
        }
    }

    async fn handle(self: &Arc<Self>, mut erx: UnboundedReceiver<Event>,
        allocator: fn(SocketBuilder) -> Arc<dyn Socket>) {
        while let Some(event) = erx.recv().await {
            match event {
                Event::Listen(listener) => {
                    let permit = self.listeners
                        .clone()
                        .acquire_owned()
                        .await
                        .unwrap();
                    listener.bind(permit, self.sockets.clone(), allocator).await;
                },
                Event::Connect(addr, socket) => {
                    let permit = self.listeners
                        .clone()
                        .acquire_owned()
                        .await
                        .unwrap();
                    socket.connect(addr, permit).await;
                }
            }
        }
    }

    async fn wait(self: &Arc<Self>, listeners: usize, sockets: usize) {
        while self.listeners.available_permits() != listeners {
            task::yield_now().await;
        }

        while self.sockets.available_permits() != sockets {
            task::yield_now().await;
        }
    }
}

impl Debug for DefaultComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctx = &self.context;
        write!(
            f, 
            "Component: {{ listeners:{}, sockets:{} }}", 
            ctx.listeners.available_permits(), 
            ctx.sockets.available_permits()
        )
    }
}

impl Display for DefaultComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{:?}}}", self)
    }
}

impl<L, S> Component<L, S> for DefaultComponent
where
    L: ListenerCreator + Listener,
    S: SocketCreator + Socket,
{
    fn listen(&mut self, addr: SocketAddr) -> Result<Arc<dyn Listener>, Error> {
        if self.context.listeners.available_permits() == 0 {
            return Err(Error::Module("listener available permit is not enough"));
        }

        let builder = ListenerBuilder::new(
            addr, 
            self.framer.clone(), 
            self.mtx.clone(), 
            self.stx.subscribe(),
        );
        let listener = L::new(builder);
        self.etx.send(Event::Listen(listener.clone()))?;
        Ok(listener)
    }

    fn connect(&mut self, addr: SocketAddr) -> Result<Arc<dyn Socket>, Error> {
        if self.context.sockets.available_permits() == 0 {
            return Err(Error::Module("socket available permit is not enough"));
        }

        let builder = SocketBuilder::new(
            self.framer.clone(),
            self.mtx.clone(),
            self.stx.subscribe(),
        );

        let socket = S::new(builder);
        self.etx.send(Event::Connect(addr, socket.clone()))?;
        Ok(socket)
    }

    fn dispatch(&mut self) -> bool {
        let result = self.dispatcher.as_mut().map(
            |dispatcher| {
                for _ in 0..DEFAULT_DISPATCH_COUNT {
                    match self.mrx.try_recv() {
                        Ok(message) => {
                            match message {
                                Message::ListenFatal(listener, err) => {
                                    dispatcher.listen_fatal(listener, err);
                                },
                                Message::ConnectDone(listener, socket) => {
                                    dispatcher.connect_done(listener, socket);
                                },
                                Message::ConnectAbort(socket, err) => {
                                    dispatcher.connect_abort(socket.clone(), err);
                                    dispatcher.connect_terminate(socket);
                                },
                                Message::ConnectFatal(socket, err) => {
                                    dispatcher.connect_fatal(socket, err);
                                },
                                Message::ConnectTerminate(socket) => {
                                    dispatcher.connect_terminate(socket);
                                },
                                Message::ReceiveDone(socket, bytes) => {
                                    dispatcher.receive_done(socket, bytes);
                                }
                            }
                        },
                        _ => return false,
                    }
                }
                true
            }
        );

        result.unwrap()
    }

    fn close(self) {
        let _ = self.stx.send(());
        self.wait.join().unwrap();
    }
}

impl<L, S> ComponentCreator<L, S> for DefaultComponent
where
    L: ListenerCreator + Listener,
    S: SocketCreator + Socket,
{
    fn new(builder: ComponentBuilder) -> Self {
        let (mtx, mrx) = mpsc::channel(builder.messages);
        let (etx, erx) = mpsc::unbounded_channel();
        let (stx, _) = broadcast::channel(1);

        let context = ThreadContext::new(
            builder.listeners,
            builder.sockets,
        );
        let wait = context.start(
            erx,
            stx.subscribe(),
            builder.listeners,
            builder.sockets,
            |builder| S::new(builder) as Arc<dyn Socket>,
        );

        Self {
            framer: builder.framer,
            mtx,
            mrx,
            etx,
            stx,
            dispatcher: builder.dispatcher,
            context,
            wait,
        }
    }
}