use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    task::{
        Context,
        Poll::{self, Pending, Ready},
    },
};

use crate::future::AsyncTcpStream;

pub struct AsyncTcpListener {
    pub listener: TcpListener,
}

impl AsyncTcpListener {
    pub fn from(listener: TcpListener) -> io::Result<Self> {
        listener.set_nonblocking(true)?;
        return Ok(Self { listener });
    }

    pub async fn accept(self: &Self) -> io::Result<(AsyncTcpStream, SocketAddr)> {
        let (stream, addr) = AcceptFuture {
            listener: &self.listener,
        }
        .await?;

        let res = (AsyncTcpStream::from(stream)?, addr);
        return Ok(res);
    }
}

pub struct AcceptFuture<'t> {
    listener: &'t TcpListener,
}
impl<'t> Future for AcceptFuture<'t> {
    type Output = io::Result<(TcpStream, SocketAddr)>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.accept() {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                cx.waker().clone().wake();
                return Pending;
            }
            res => Ready(res),
        }
    }
}
