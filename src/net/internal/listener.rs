use crate::Error;
use crate::Manager;
use crate::Parser;
use crate::Message;
use crate::Socket;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::OwnedSemaphorePermit;
use std::sync::Arc;
use std::sync::Weak;
use std::net::SocketAddr;

pub enum State {
    Binding,
    Done(TcpListener),
    Fatal,
}

pub struct Listener {
    state: RwLock<State>,
    parser: Arc<Parser>,
    owner: Weak<Manager>,
    dispatcher: mpsc::UnboundedSender<Message>,
    shutdown: broadcast::Receiver<()>,
}

impl Listener {
    pub fn new(manager: &Arc<Manager>, parser: Arc<Parser>, 
        dispatcher: mpsc::UnboundedSender<Message>, 
        shutdown: broadcast::Receiver<()>) -> Arc<Listener> {
        Arc::new(Listener {
            parser,
            dispatcher,            
            shutdown,
            owner: Arc::downgrade(manager),
            state: RwLock::new(State::Binding),
        })
    }

    pub async fn bind(self: &Arc<Listener>, permit: OwnedSemaphorePermit, 
        addr: SocketAddr, semaphore: Arc<Semaphore>) {
        let result = TcpListener::bind(addr).await;
        if let Err(err) = result {
            let mut state = self.state.write().await;
            *state = State::Fatal;

            let message = Message::ListenFatal(self.clone(), Error::Io(err));
            let _ = self.dispatcher.send(message);
            return;
        }

        let mut state = self.state.write().await;
        *state = State::Done(result.unwrap());

        self.start(permit, semaphore);
    }

    fn start(self: &Arc<Listener>, permit: OwnedSemaphorePermit, semaphore: Arc<Semaphore>) {
        let listener = self.clone();
        let mut shutdown = self.shutdown.resubscribe();

        tokio::spawn(async move {
            select! {
                _ = listener.run(semaphore) => (),
                _ = shutdown.recv() => {
                    drop(permit);
                    return;
                }
            }
        });
    }

    async fn run(self: &Arc<Listener>, semaphore: Arc<Semaphore>) {
        let state = self.state.read().await;

        if let State::Done(listener) = &*state {
            loop {
                let result = listener.accept().await;
                if result.is_err() {
                    continue;
                }

                let (stream, _) = result.unwrap();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                let socket = Socket::new(
                    self.owner.clone(),
                    self.parser.clone(),
                    self.dispatcher.clone(),
                    self.shutdown.resubscribe(),
                );
                socket.accept(permit, stream).await;

                let message = Message::ConnectDone(Some(self.clone()), socket.clone());
                let _ = self.dispatcher.send(message);
            }
        }
    }
}