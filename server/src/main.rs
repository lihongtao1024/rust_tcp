use tcp::Error;
use tcp::Dispatcher;
use tcp::Listener;
use tcp::Socket;
use tcp::Builder;
use tcp::Component;
use tcp::ServerComponent;
use tcp::DefaultListener;
use tcp::DefaultSocket;
use tcp::DefaultComponent;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::sync::broadcast::error::TryRecvError;
use bytes::Bytes;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::mem;
use std::time::Duration;

struct Manager {
}

impl Manager {
    fn new() -> Self {
        Self {  }
    }
}

impl Dispatcher for Manager {
    fn listen_fatal(&mut self, _listener: Arc<dyn Listener>, _err: Error) {
        //println!("listen_fatal: [{}, {}]", _listener, _err);
    }

    fn connect_fatal(&mut self, _socket: Arc<dyn Socket>, _err: Error) {
        //println!("connect_fatal: [{}, {}]", _socket, _err);
    }

    fn connect_done(&mut self, _listener: Option<Arc<dyn Listener>>, _socket: Arc<dyn Socket>) {        
        /*match _listener {
            Some(listener) => {
                println!("connect_done: [{}, {}]", listener, _socket);
                //listener.close();
            },
            _ => {
                println!("connect_done: [None, {}]", _socket);
            },
        };
        //_socket.disconnect();
        */
        
    }

    fn receive_done(&mut self, _socket: Arc<dyn Socket>, _bytes: Bytes) {
        //println!("receive_done: [{}, {}]", _socket, _bytes.len());
        let _ = _socket.send(_bytes);
    }

    fn connect_abort(&mut self, _socket: Arc<dyn Socket>, _err: Error) {
        //println!("connect_abort: [{}, {}]", _socket, _err);
    }

    fn connect_terminate(&mut self, _socket: Arc<dyn Socket>) {
        //println!("connect_terminate: [{}]", _socket);
    }
}

fn start(mut srx: BroadcastReceiver<()>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut mgr = Manager::new();
        let dispatcher = unsafe {
            mem::transmute::<&mut dyn Dispatcher, &'static mut dyn Dispatcher>(&mut mgr)
        };
        let mut tcp = Builder::default()
            .listener(1)
            .sockets(3000)
            .messages(7500)
            .events(1500)
            .socket_events(128)
            .dispatchs(16)
            .dispatcher(dispatcher)
            .build_server::<DefaultComponent, DefaultListener, DefaultSocket>();
        let _ = "127.0.0.1:6668".parse().and_then(|addr| {
            let _ = tcp.listen(addr);
            Ok(())
        });

        let mut busy;
        let duration = Duration::from_millis(1);
        loop {
            match srx.try_recv() {
                Err(TryRecvError::Empty) => (),
                _ => {
                    tcp.close();
                    break;
                },
            }

            busy = tcp.dispatch();
            if !busy {
                thread::sleep(duration);
            }
        }
    })
}

fn main() {
    let (stx, _) = broadcast::channel(1);
    let running = start(stx.subscribe());

    thread::spawn(move || {
        let mut reader = BufReader::new(io::stdin()).lines();
        while let Ok(line) = reader.next().unwrap() {
            if line.eq("stop") {
                drop(stx);
                running.join().unwrap();
                return;
            }
        }
    })
    .join()
    .unwrap();    
}