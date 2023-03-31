use crate::Error;
use std::mem;
use std::sync::Arc;
use std::io::Cursor;
use bytes::Buf;
use bytes::BytesMut;

pub enum Frame {
    Fatal,
    Continue,
    Success(u32),
}

pub trait Framer: Send + Sync {
    fn parse(self: Arc<Self>, cursor: &mut Cursor<&BytesMut>) -> Frame;
    fn check(self: Arc<Self>, cursor: &mut Cursor<&BytesMut>) -> Result<u32, Error> {
        match self.parse(cursor) {
            Frame::Success(len) => Ok(len),
            Frame::Continue => Ok(0),
            Frame::Fatal => Err(Error::Module("a fatal error occurred while parsing the frame")),
        }
    }
}

#[derive(Default)]
pub(crate) struct DefaultFramer;

impl Framer for DefaultFramer {
    fn parse(self: Arc<Self>, cursor: &mut Cursor<&BytesMut>) -> Frame {
        let len = cursor.get_ref().len();
        if len < mem::size_of::<u32>() {
            return Frame::Continue;
        }
    
        let size = cursor.get_u32_le() as usize;
        if len < size {
            return Frame::Continue;
        }
    
        Frame::Success(size as u32)
    }
}