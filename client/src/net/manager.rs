use tcp::Error;
use tcp::Dispatcher;
use tcp::Listener;
use tcp::Socket;
use tcp::Builder;
use tcp::Component;
use tcp::DefaultComponent;
use tcp::DefaultListener;
use tcp::DefaultSocket;
use std::net::SocketAddr;
use std::mem;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::mpsc::Receiver as MpscReceiver;

pub struct StartBuilder {
    pub(crate) addr: SocketAddr,
    pub(crate) size: usize,
    pub(crate) conn: usize,
}

pub enum UiEvent {
    Start(StartBuilder),
    Stop,
}

pub(crate) struct Manager {

}

impl Manager {
    fn new() -> Self {
        Self {  }
    }
}

impl Dispatcher for Manager {
    fn listen_fatal(&mut self, _listener: Arc<dyn Listener>, _err: Error) {
    }

    fn connect_fatal(&mut self, _socket: Arc<dyn Socket>, _err: Error) {
    }

    fn connect_done(&mut self, _listener: Option<Arc<dyn Listener>>, _socket: Arc<dyn Socket>) {
    }

    fn receive_done(&mut self, _socket: Arc<dyn Socket>, _bytes: bytes::Bytes) {
    }

    fn connect_abort(&mut self, _socket: Arc<dyn Socket>, _err: Error) {
    }

    fn connect_terminate(&mut self, _socket: Arc<dyn Socket>) {
    }
}

pub fn start(srx: MpscReceiver<UiEvent>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut manager = Manager::new();
        let mut tcp = None;

        while let Ok(event) = srx.recv() {
            match event {
                UiEvent::Start(builder) => {
                    let dispatcher = unsafe {
                        mem::transmute::<
                            &mut dyn Dispatcher,
                            &'static mut dyn Dispatcher
                        >(&mut manager)
                    };

                    tcp = Some(
                        Builder::default()
                            .listener(0)
                            .sockets(builder.conn)
                            .dispatcher(dispatcher)
                            .build::<
                                DefaultComponent,
                                DefaultListener,
                                DefaultSocket
                            >()
                    );
                },
                UiEvent::Stop => {
                    tcp
                        .take()
                        .map(|tcp| tcp.close());
                }
            }
        }
    })
}