use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::net::TcpStream;
use tokio::sync::OwnedSemaphorePermit;
use std::sync::Arc;
use std::net::SocketAddr;
use std::fmt::Debug;
use std::fmt::Display;

#[async_trait]
pub trait AsyncSocket {
    async fn accept(self: Arc<Self>, permit: OwnedSemaphorePermit, stream: TcpStream);
    async fn connect(self: Arc<Self>, addr: SocketAddr, permit: OwnedSemaphorePermit);
}

pub trait Socket: AsyncSocket + Send + Sync + Debug + Display + 'static {
    fn send(self: Arc<Self>, bytes: Bytes) -> Result<(), Error>;
    fn disconnect(self: Arc<Self>);
    fn local_addr(self: Arc<Self>) -> Option<SocketAddr>;
    fn peer_addr(self: Arc<Self>) -> Option<SocketAddr>;
    fn set_tag(self: Arc<Self>, tag: usize);
    fn tag(self: Arc<Self>) -> Option<usize>;
}