use bytes::BytesMut;
use std::io::Cursor;
use crate::Error;
use crate::Result;

pub type ExtParser = Box<dyn Parser + Send + Sync>;

pub enum Frame {
    Fatal,
    Continue,
    Success(u32),
}

pub trait Parser {
    fn parse(&self, data: &mut Cursor<&BytesMut>) -> Frame;
}

impl Frame {
    pub fn check(cursor: &mut Cursor<&BytesMut>, parser: &ExtParser) -> Result<u32> {
        match parser.parse(cursor) {
            Frame::Success(len) => Ok(len),
            Frame::Continue => Ok(0),
            Frame::Fatal => Err(Error::Fatal("a fatal error occurred while parsing the frame".to_string())),
        }
    }
}