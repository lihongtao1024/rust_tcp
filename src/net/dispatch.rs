use bytes::Bytes;
use std::sync::Arc;
use std::future::Future;
use crate::Error;
use crate::socket;
use crate::listener;

pub struct Dispatcher<FA, FB, FC, FD, FE, FF>
where
    FA: Future<Output = ()> + Sync + Send + 'static,
    FB: Future<Output = ()> + Sync + Send + 'static,
    FC: Future<Output = ()> + Sync + Send + 'static,
    FD: Future<Output = ()> + Sync + Send + 'static,
    FE: Future<Output = ()> + Sync + Send + 'static,
    FF: Future<Output = ()> + Sync + Send + 'static {
        pub bound_message: fn (Arc<listener::Context>, Error) -> FA,
        pub connected_message: fn (Arc<listener::Context>, Arc<socket::Context>) -> FB,
        pub received_message: fn (Arc<socket::Context>, Bytes) -> FC,
        pub error_message: fn (Arc<socket::Context>, Error) -> FD,
        pub terminated_message: fn (Arc<socket::Context>) -> FE,
        pub fatal_message: fn (Error) -> FF,
}