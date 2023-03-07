#[derive(Clone)]
pub struct Config {
    pub send_size: usize,
    pub recv_size: usize,
    pub listener_limit: usize,
    pub socket_limit: usize,    
    pub message_limit: usize,
}

impl Config {
    pub fn new(send_size: usize, recv_size: usize, listener_limit: usize, 
        socket_limit: usize, message_limit: usize) -> Self {
        Self { send_size, recv_size, listener_limit, socket_limit, message_limit }
    }
}