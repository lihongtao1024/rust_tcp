pub struct Config {
    pub socket_recv_buf_size: usize,
    pub socket_send_request_limit: usize,
    pub bind_listener_limit: usize,
    pub conn_socket_limit: usize,
    pub dispatch_queue_limit: usize,
}

impl Config {
    pub fn new(socket_recv_buf_size: usize, socket_send_request_limit: usize,
        bind_listener_limit: usize, conn_socket_limit: usize, dispatch_queue_limit: usize) -> Self {
        Self { 
            socket_recv_buf_size,
            socket_send_request_limit,
            bind_listener_limit,
            conn_socket_limit,
            dispatch_queue_limit,
        }
    }
}