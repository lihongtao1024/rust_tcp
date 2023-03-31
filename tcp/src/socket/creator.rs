use crate::SocketBuilder;
use std::sync::Arc;

pub trait Creator {
    fn new(builder: SocketBuilder) -> Arc<Self>;
}