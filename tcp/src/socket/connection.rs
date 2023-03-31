use crate::Error;
use crate::Framer;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use bytes::Bytes;
use bytes::BytesMut;
use std::io::Cursor;
use std::sync::Arc;

pub(crate) struct ConnectionReader<'a> {
    buf: BytesMut,
    stream: &'a mut OwnedReadHalf,
    framer: &'a Arc<dyn Framer>,
}

pub(crate) struct ConnectionWriter<'a> {
    stream: &'a mut OwnedWriteHalf,
}

impl<'a> ConnectionReader<'a> {
    pub(crate) fn new(size: usize, stream: &'a mut OwnedReadHalf, 
        framer: &'a Arc<dyn Framer>) -> Self {
        Self { 
            buf: BytesMut::with_capacity(size),
            stream,
            framer,
        }
    }

    pub(crate) async fn read_frame(&mut self) -> Result<Option<Bytes>, Error> {
        loop {
            if let Some(bytes) = self.parse_frame()? {
                return Ok(Some(bytes));
            }

            if 0 == (*self.stream).read_buf(&mut self.buf).await? {
                if self.buf.is_empty() {
                    return Ok(None)
                } else {
                    return Err(Error::Module("connection reset by peer"))
                }
            }
        }
    }

    pub(crate) fn parse_frame(&mut self) -> Result<Option<Bytes>, Error> {
        let mut cursor = Cursor::new(&self.buf);
        match self.framer.clone().check(&mut cursor) {
            Ok(0) => Ok(None),
            Ok(len) => {
                let frame = self.buf.split_to(len as usize).freeze();
                Ok(Some(frame))
            },
            _ => Ok(None),
        }
    }
}

impl<'a> ConnectionWriter<'a> {
    pub(crate) fn new(stream: &'a mut OwnedWriteHalf) -> Self {
        Self { stream }
    }

    pub(crate) async fn write_frame(&mut self, bytes: Bytes) -> Result<(), Error> {
        self.stream.write_all(bytes.as_ref()).await?;
        Ok(())
    }
}