use crate::Listener;
use crate::Socket;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;

pub enum Event {
    Listen(SocketAddr, Arc<Listener>),
    Connect(SocketAddr, Arc<Socket>),
    Send(Arc<Socket>, Bytes),
}