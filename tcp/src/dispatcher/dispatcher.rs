use crate::Error;
use crate::Listener;
use crate::Socket;
use std::sync::Arc;
use bytes::Bytes;

pub trait Dispatcher {
    fn listen_fatal(&mut self, listener: Arc<dyn Listener>, err: Error);
    fn connect_fatal(&mut self, socket: Arc<dyn Socket>, err: Error);
    fn connect_done(&mut self, listener: Option<Arc<dyn Listener>>, socket: Arc<dyn Socket>);
    fn receive_done(&mut self, socket: Arc<dyn Socket>, bytes: Bytes);
    fn connect_abort(&mut self, socket: Arc<dyn Socket>, err: Error);
    fn connect_terminate(&mut self, socket: Arc<dyn Socket>);
}