use crate::Error;
use crate::Result;
use crate::Frame;
use crate::Parser;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use bytes::Bytes;
use bytes::BytesMut;
use std::io::Cursor;

pub struct ConnectionReader<'a> {
    buf: BytesMut,
    stream: &'a mut ReadHalf<TcpStream>,
    parser: &'a Parser,
}

pub struct ConnectionWriter<'a> {
    stream: &'a mut WriteHalf<TcpStream>,
}

impl<'a> ConnectionReader<'a> {
    pub fn new(size: usize, stream: &'a mut ReadHalf<TcpStream>, parser: &'a Parser) -> Self {
        Self { 
            buf: BytesMut::with_capacity(size),
            stream,
            parser,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Bytes>> {
        loop {
            if let Some(bytes) = self.parse_frame()? {
                return Ok(Some(bytes));
            }

            if 0 == (*self.stream).read_buf(&mut self.buf).await? {
                if self.buf.is_empty() {
                    return Ok(None)
                } else {
                    return Err(Error::Fatal("connection reset by peer".to_string()))
                }
            }
        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Bytes>> {
        let mut cursor = Cursor::new(&self.buf);
        match Frame::check(&mut cursor, &self.parser) {
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
    pub fn new(stream: &'a mut WriteHalf<TcpStream>) -> Self {
        Self { stream }
    }

    pub async fn write_frame(&mut self, bytes: Bytes) -> Result<()> {
        self.stream.write_all(bytes.as_ref()).await?;
        Ok(())
    }
}