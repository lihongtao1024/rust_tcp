use std::future::Future;
use crate::Dispatcher;
use crate::ExtParser;

pub struct Config<F1, F2, F3, F4, F5, F6>
where
    F1: Future<Output = ()> + Sync + Send + 'static,
    F2: Future<Output = ()> + Sync + Send + 'static,
    F3: Future<Output = ()> + Sync + Send + 'static,
    F4: Future<Output = ()> + Sync + Send + 'static,
    F5: Future<Output = ()> + Sync + Send + 'static,
    F6: Future<Output = ()> + Sync + Send + 'static {
    pub socket_send_buf_size: usize,
    pub socket_recv_buf_size: usize,
    pub socket_send_request_limit: usize,
    pub bind_listener_limit: usize,
    pub conn_socket_limit: usize,    
    pub dispatch_queue_limit: usize,
    pub dispatch_instance: Dispatcher<F1, F2, F3, F4, F5, F6>,
    pub parser_instance: ExtParser,
}

impl<F1, F2, F3, F4, F5, F6> Config<F1, F2, F3, F4, F5, F6>
where 
    F1: Future<Output = ()> + Sync + Send + 'static,
    F2: Future<Output = ()> + Sync + Send + 'static,
    F3: Future<Output = ()> + Sync + Send + 'static,
    F4: Future<Output = ()> + Sync + Send + 'static,
    F5: Future<Output = ()> + Sync + Send + 'static,
    F6: Future<Output = ()> + Sync + Send + 'static {
    pub fn new(socket_send_buf_size: usize, socket_recv_buf_size: usize, 
        socket_send_request_limit: usize, bind_listener_limit: usize, 
        conn_socket_limit: usize, dispatch_queue_limit: usize,
        dispatch_instance: Dispatcher<F1, F2, F3, F4, F5, F6>, 
        parser_instance: ExtParser) -> Self {
        Self { 
            socket_send_buf_size,
            socket_recv_buf_size,
            socket_send_request_limit,
            bind_listener_limit,
            conn_socket_limit,
            dispatch_queue_limit,
            dispatch_instance,
            parser_instance,
        }
    }
}