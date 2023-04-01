#![feature(sync_unsafe_cell)]

mod error;
mod framer;
mod message;
mod dispatcher;
mod listener;
mod socket;
mod component;

pub(crate) use message::message::Message;
pub(crate) use framer::framer::DefaultFramer;
pub(crate) use listener::listener::AsyncListener;
pub(crate) use listener::builder::Builder as ListenerBuilder;
pub(crate) use listener::creator::Creator as ListenerCreator;
pub(crate) use socket::socket::AsyncSocket;
pub(crate) use socket::builder::Builder as SocketBuilder;
pub(crate) use socket::creator::Creator as SocketCreator;
pub(crate) use socket::connection::ConnectionReader;
pub(crate) use socket::connection::ConnectionWriter;
pub(crate) use component::builder::Builder as ComponentBuilder;
pub(crate) use component::creator::Creator as ComponentCreator;

pub use error::error::Error;
pub use framer::framer::Framer;
pub use dispatcher::dispatcher::Dispatcher;
pub use listener::listener::Listener;
pub use socket::socket::Socket;
pub use component::builder::Builder;
pub use component::component::Component;
pub use listener::default_listener::DefaultListener;
pub use socket::default_socket::DefaultSocket;
pub use component::default_component::DefaultComponent;