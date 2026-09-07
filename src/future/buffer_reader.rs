use std::{
    io::{self},
    ops::DerefMut,
};

use crate::future::AsyncRead;

const DEFAULT_BUFFER_SIZE: usize = 512;
pub struct AsyncBufferReader<AR: AsyncRead> {
    reader: AR,
    buf: Box<[u8; DEFAULT_BUFFER_SIZE]>,
    start: usize,
    end: usize,
}

impl<AR: AsyncRead> From<AR> for AsyncBufferReader<AR> {
    fn from(reader: AR) -> Self {
        return Self {
            reader,
            buf: Box::new([0; DEFAULT_BUFFER_SIZE]),
            start: 0,
            end: 0,
        };
    }
}

impl<AR: AsyncRead> AsyncRead for AsyncBufferReader<AR> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // read from internal buffer before reading from stream
        if self.start < self.end {
            let available = &self.buf[self.start..self.end];
            let n = std::cmp::min(available.len(), buf.len());
            buf[..n].copy_from_slice(&available[..n]);
            self.start += n;
            return Ok(n);
        }

        match self.reader.read(buf).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Err(e);
            }
        }
    }
}
pub trait BufferRead: AsyncRead {
    #[allow(async_fn_in_trait)]
    async fn read_until(self: &mut Self, expected: u8, buf: &mut Vec<u8>) -> io::Result<()>;
}

impl<AR: AsyncRead> AsyncBufferReader<AR> {
    fn try_read_until(self: &mut Self, buf: &mut Vec<u8>, delimiter: u8, start: usize, end: usize) -> bool {
        let internal_buf = self.buf.deref_mut();

        let remaining = &internal_buf[start..end];
        if let Some(pos) = remaining.iter().position(|b| *b == delimiter) {
            buf.extend_from_slice(&remaining[..pos]);
            //advance position and skip delimiter
            self.start = start + pos + 1;
            self.end = end;
            return true;
        }

        buf.extend_from_slice(&remaining);
        return false;
    }
    pub fn take_reader(self) -> AR {
        return self.reader;
    }
    pub fn as_mut_reader(&mut self) -> &mut AR {
        return &mut self.reader;
    }
}
impl<AR: AsyncRead> BufferRead for AsyncBufferReader<AR> {
    /// reads using passed buffer until `expected` is reached or EOF
    /// returns the size of bytes that has been written
    async fn read_until(self: &mut Self, delimiter: u8, buf: &mut Vec<u8>) -> io::Result<()> {
        //read from internal buffer before reading from stream
        if self.try_read_until(buf, delimiter, self.start, self.end) {
            return Ok(());
        }

        loop {
            let internal_buf = self.buf.deref_mut();
            let n = self.reader.read(internal_buf).await?;
            if n == 0 {
                break;
            }

            if self.try_read_until(buf, delimiter, 0, n) {
                break;
            }
        }

        return Ok(());
    }
}
