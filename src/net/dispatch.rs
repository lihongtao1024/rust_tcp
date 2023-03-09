use bytes::Bytes;
use std::sync::Arc;
use std::future::Future;
use crate::Error;
use crate::socket;
use crate::listener;

pub struct Dispatcher<F1, F2, F3, F4, F5, F6>
where
    F1: Future<Output = ()> + Sync + Send + 'static,
    F2: Future<Output = ()> + Sync + Send + 'static,
    F3: Future<Output = ()> + Sync + Send + 'static,
    F4: Future<Output = ()> + Sync + Send + 'static,
    F5: Future<Output = ()> + Sync + Send + 'static,
    F6: Future<Output = ()> + Sync + Send + 'static {
        pub bound_message: fn (Arc<listener::Context>, Error) -> F1,
        pub connected_message: fn (Arc<listener::Context>, Arc<socket::Context>) -> F2,
        pub received_message: fn (Arc<socket::Context>, Bytes) -> F3,
        pub error_message: fn (Arc<socket::Context>, Error) -> F4,
        pub terminated_message: fn (Arc<socket::Context>) -> F5,
        pub fatal_message: fn (Error) -> F6,
}