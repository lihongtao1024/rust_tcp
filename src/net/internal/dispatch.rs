use crate::Error;
use crate::Listener;
use crate::Socket;
use bytes::Bytes;
use std::sync::Arc;

pub enum Message {
    ListenFatal(Arc<Listener>, Error),
    ConnectFatal(Arc<Socket>, Error),
    ConnectDone(Option<Arc<Listener>>, Arc<Socket>),
    ReceiveDone(Arc<Socket>, Bytes),
    ConnectAbort(Arc<Socket>, Error),
    ConnectTerminate(Arc<Socket>),
}

pub type ListenFatal = fn (Arc<Listener>, Error);
pub type ConnectFatal = fn (Arc<Socket>, Error);
pub type ConnectDone = fn (Option<Arc<Listener>>, Arc<Socket>);
pub type ReceiveDone = fn (Arc<Socket>, Bytes);
pub type ConnectAbort = fn (Arc<Socket>, Error);
pub type ConnectTerminate = fn (Arc<Socket>);

pub struct Dispatcher {
    pub listen_fatal: ListenFatal,
    pub connect_fatal: ConnectFatal,
    pub connect_done: ConnectDone,
    pub receive_done: ReceiveDone,
    pub connect_abort: ConnectAbort,
    pub connect_terminate: ConnectTerminate,
}

impl Dispatcher {
    pub fn build(listen_fatal: ListenFatal, connect_fatal: ConnectFatal,
        connect_done: ConnectDone, receive_done: ReceiveDone, 
        connect_abort: ConnectAbort, connect_terminate: ConnectTerminate) -> Self {
            Self {
                listen_fatal,
                connect_fatal,
                connect_done,
                receive_done,
                connect_abort,
                connect_terminate,
            }
        }
}