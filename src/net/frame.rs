use bytes::BytesMut;
use std::io::Cursor;
use crate::Error;
use crate::Result;

pub enum Frame {
    Fatal,
    Continue,
    Success(u32),
}

pub struct Parser {
    pub parse: fn (&mut Cursor<&BytesMut>) -> Frame,
}

impl Frame {
    pub fn check(cursor: &mut Cursor<&BytesMut>, parser: &Parser) -> Result<u32> {
        match (parser.parse)(cursor) {
            Frame::Success(len) => Ok(len),
            Frame::Continue => Ok(0),
            Frame::Fatal => Err(Error::Fatal("a fatal error occurred while parsing the frame".to_string())),
        }
    }
}