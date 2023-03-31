use crate::ListenerCreator;
use crate::SocketCreator;
use crate::ComponentBuilder;
use crate::listener::listener::Listener;
use crate::socket::socket::Socket;

pub trait Creator<L, S>
where
    L: ListenerCreator + Listener,
    S: SocketCreator + Socket,
{
    fn new(builder: ComponentBuilder) -> Self;
}