pub struct Config {
    pub bind_listener_limit: usize,
    pub conn_socket_limit: usize,
}

impl Config {
    pub fn new(bind_listener_limit: usize, conn_socket_limit: usize) -> Self {
        Self { bind_listener_limit, conn_socket_limit }
    }
}