use crate::ListenerBuilder;
use std::sync::Arc;

pub trait Creator {
    fn new(builder: ListenerBuilder) -> Arc<Self>;
}