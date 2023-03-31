use crate::Listener;
use crate::Socket;
use crate::Error;
use std::sync::Arc;
use bytes::Bytes;

pub(crate) enum Message {
    ListenFatal(Arc<dyn Listener>, Error),
    ConnectFatal(Arc<dyn Socket>, Error),
    ConnectDone(Option<Arc<dyn Listener>>, Arc<dyn Socket>),
    ReceiveDone(Arc<dyn Socket>, Bytes),
    ConnectAbort(Arc<dyn Socket>, Error),
    ConnectTerminate(Arc<dyn Socket>),
}