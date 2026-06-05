use std::{io, net::TcpStream};

use crate::future::{AsyncRead, ReadFuture};

pub struct AsyncTcpStream {
    stream: TcpStream,
}

impl AsyncTcpStream {
    pub fn from(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        return Ok(Self { stream });
    }
}
impl AsyncRead for AsyncTcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        ReadFuture {
            buf,
            reader: &mut self.stream,
        }
        .await
    }
}
