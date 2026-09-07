use std::io;

use crate::future::{AsyncBufferReader, AsyncRead};

pub struct SizedBodyReader<AR: AsyncRead> {
    raw: AsyncBufferReader<AR>,
    remaining: usize,
}

impl<AR: AsyncRead> SizedBodyReader<AR> {
    pub fn new(raw: AsyncBufferReader<AR>, len: usize) -> Self {
        Self { raw, remaining: len }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Ok(0);
        }

        let to_read = buf.len().min(self.remaining);
        let n = self.raw.read(&mut buf[..to_read]).await?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed before content-length satisfied",
            ));
        }

        self.remaining -= n;
        Ok(n)
    }

    pub fn into_raw(self) -> AsyncBufferReader<AR> {
        self.raw
    }

    pub fn into_raw_mut(&mut self) -> &mut AsyncBufferReader<AR> {
        &mut self.raw
    }
}
