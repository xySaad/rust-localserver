use std::{
    io::{self, Write},
    pin::Pin,
    task::{
        Context,
        Poll::{self, Pending, Ready},
    },
};
/// NOTE: the writer should be set to non-blocking mode otherwise this will block, in other words `.write` should return `io::ErrorKind::WouldBlock`
pub struct WriteFuture<'t, 'b, R: Write> {
    pub writer: &'t mut R,
    pub buf: &'b [u8],
}

impl<R: Write> Future for WriteFuture<'_, '_, R> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.writer.write(this.buf) {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                cx.waker().clone().wake();
                return Pending;
            }
            v => Ready(v),
        }
    }
}
pub trait AsyncWrite {
    #[allow(async_fn_in_trait)]
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
}
impl<R: AsyncWrite> AsyncWrite for &mut R {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return (*self).write(buf).await;
    }
}
