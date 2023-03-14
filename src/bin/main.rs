use tcp::Error;
use tcp::Frame;
use tcp::Config;
use tcp::Parser;
use tcp::Manager;
use tcp::Listener;
use tcp::Socket;
use tcp::Dispatcher;
use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use std::io::BufRead;
use std::mem;
use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use std::io::BufReader;
use std::io::stdin;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error;

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

fn listen_fatal(_: Arc<Listener>, _: Error) {

}

fn connect_fatal(_: Arc<Socket>, _: Error) {

}

fn connect_done(_: Option<Arc<Listener>>, _: Arc<Socket>) {
    
}

fn receive_done(socket: Arc<Socket>, data: Bytes) {
    //println!("receive data: {}", data.len());
    socket.send_to(data);
}

fn connect_abort(_: Arc<Socket>, _: Error) {

}

fn connect_terminate(_: Arc<Socket>) {

}

fn main() {
    let config = Config::new(
        16, 
        3000,
    );

    let parser = Parser { parse };
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_rx1 = shutdown_tx.subscribe();
    let mut shutdown_rx2 = shutdown_tx.subscribe();
    let running = thread::spawn(move || {
        let dispatch = Dispatcher::build(
            listen_fatal,
            connect_fatal,
            connect_done,
            receive_done,
            connect_abort,
            connect_terminate,
        );
        let mgr = Manager::new(
            config, 
            dispatch, 
            parser, 
            shutdown_rx1
        );
        let _ = mgr.listen("127.0.0.1:6668".parse().unwrap());

       loop {
            match shutdown_rx2.try_recv() {
                Err(error::TryRecvError::Empty) => (),
                _ => break,
            }

            mgr.dispatch(10);
        }                
    });

    let console = thread::spawn(move || {
        let mut reader = BufReader::new(stdin()).lines();
        while let Ok(line) = reader.next().unwrap() {
            if line.eq("stop") {
                drop(shutdown_tx);
                return;
            }
        }
    });  

    let _ = console.join();
    let _ = running.join();
}