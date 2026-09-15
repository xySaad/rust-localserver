use std::{
    io,
    net::{Shutdown, TcpStream},
};

use crate::future::{AsyncRead, AsyncWrite, ReadFuture, WriteFuture};

pub struct AsyncTcpStream {
    stream: TcpStream,
}

impl AsyncTcpStream {
    pub fn from(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        return Ok(Self { stream });
    }
    pub fn shutdown(&self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
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

impl AsyncWrite for AsyncTcpStream {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        WriteFuture {
            buf,
            writer: &mut self.stream,
        }
        .await
    }
}
