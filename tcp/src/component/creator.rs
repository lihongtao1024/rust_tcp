use crate::SocketCreator;
use crate::ComponentBuilder;
use crate::socket::socket::Socket;

pub trait ComponentCreator<S>
where
    S: SocketCreator + Socket,
{
    fn new(builder: ComponentBuilder) -> Self;
}