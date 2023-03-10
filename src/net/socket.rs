use std::sync::Arc;
use std::future::Future;
use std::net::SocketAddr;
use bytes::Bytes;
use tokio::select;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::net::tcp::ReadHalf;
use tokio::net::tcp::WriteHalf;
use tokio::sync::mpsc::Sender;
use tokio::sync::broadcast::Receiver;
use tokio::sync::OwnedSemaphorePermit;
use crate::Config;
use crate::Message;
use crate::Parser;
use crate::ConnectionReader;
use crate::ConnectionWriter;

pub struct Socket {
    peer_addr: SocketAddr,
    local_addr: SocketAddr,
    internal: Arc<Context>,
}

pub struct Context {
    request_sender: mpsc::Sender<Bytes>,
    dispatch_sender: mpsc::Sender<Message>,
}

impl Socket {
    pub fn new(mut stream: TcpStream, permit: OwnedSemaphorePermit,
        config: Arc<Config>, parser: Arc<Parser>, dispatch_sender: Sender<Message>, 
        mut shutdown: Receiver<()>) -> Self {        
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();
        let (request_sender, mut request_receiver) = mpsc::channel(
            config.socket_send_request_limit
        );
        let context = Arc::new(Context { request_sender, dispatch_sender });

        let running_context = context.clone();
        tokio::spawn(async move {
            let (mut reader, mut writer) = stream.split();
            select! {
                _ = running_context.read(&mut reader, &config, &parser) => (),
                _ = running_context.write(&mut writer, &mut request_receiver) => (),
                _ = shutdown.recv() => {
                    drop(permit);
                }
            }
        });

        Self { peer_addr, local_addr, internal: context }
    }

    pub fn context(&self) -> &Arc<Context> {
        &self.internal
    }

    pub fn equal(&self, context: &Arc<Context>) -> bool {
        Arc::ptr_eq(&self.internal, context)
    }

    pub fn local_addr(&self) -> &SocketAddr {
        &self.local_addr
    }

    pub fn peer_addr(&self) -> &SocketAddr {
        &self.peer_addr
    }

    pub async fn send(&self, bytes: Bytes) {
        self.internal.send(bytes).await;
    }
}

impl Context {
    pub async fn send(&self, bytes: Bytes) {
        let _ = self.request_sender.send(bytes).await;
    }

    async fn read<'a>(self: &Arc<Context>, stream: &'a mut ReadHalf<'a>, 
        config: &Config, parser: &'a Parser) {
        let mut connection = ConnectionReader::new(
            config.socket_recv_buf_size, 
            stream, 
            parser,
        );
        loop {
            let result = connection.read_frame().await;
            if let Err(err) = result {
                Message::send_error_message(
                    &self.dispatch_sender, 
                    self, 
                    err
                ).await;
                return;
            }
    
            if let Some(bytes) = result.unwrap() {
                Message::send_received_message(
                    &self.dispatch_sender, 
                    self, 
                    bytes
                ).await;
            } else {
                Message::send_terminated_message(
                    &self.dispatch_sender, 
                    self,
                ).await;
                return;
            }
        }
    }

    async fn write<'a>(self: &Arc<Context>, stream: &'a mut WriteHalf<'a>, 
        request_receiver: &'a mut mpsc::Receiver<Bytes>) {
        let mut connection = ConnectionWriter::new(
            stream,
            request_receiver
        );

        loop {
            let result = connection.write_frame().await;
            if let Err(err) = result {
                Message::send_error_message(
                    &self.dispatch_sender, 
                    self, 
                    err
                ).await;
                return;
            }
        }
    }
}