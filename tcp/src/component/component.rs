use crate::Error;
use crate::Listener;
use crate::ListenerCreator;
use crate::Socket;
use crate::SocketCreator;
use std::net::SocketAddr;
use std::sync::Arc;
use std::fmt::Debug;
use std::fmt::Display;

pub trait Component<L, S>: Debug + Display
where
    L: ListenerCreator + Listener,
    S: SocketCreator + Socket,
{
    fn listen(&mut self, addr: SocketAddr) -> Result<Arc<dyn Listener>, Error>;
    fn connect(&mut self, addr: SocketAddr) -> Result<Arc<dyn Socket>, Error>;
    fn dispatch(&mut self) -> bool;
    fn close(self);
}