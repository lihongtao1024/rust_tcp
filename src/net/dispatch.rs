use std::sync::Arc;
use crate::Error;
use crate::socket;
use crate::listener;

pub trait Dispatch {
    fn bound_message(&self, listener: Arc<listener::Context>, err: Error);

    fn connected_message(&self, listener: Arc<listener::Context>, socket: Arc<socket::Context>);

    fn error_message(&self, socket: Arc<socket::Context>, err: Error);

    fn fatal_message(&self, err: Error);
}