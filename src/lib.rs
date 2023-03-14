pub mod net;
pub use net::internal::error::Error;
pub use net::internal::error::Result;
pub use net::internal::config::Config;
pub use net::internal::parser::Parser;
pub use net::internal::parser::Frame;
pub use net::internal::event::Event;
pub use net::internal::dispatch::Message;
pub use net::internal::dispatch::Dispatcher;
pub use net::internal::listener::Listener;
pub use net::internal::socket::Socket;
pub use net::internal::connection::ConnectionReader;
pub use net::internal::connection::ConnectionWriter;
pub use net::manager::Manager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
    }

    #[test]
    fn test2() {
    }
}
