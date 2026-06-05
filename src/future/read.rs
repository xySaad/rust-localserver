use std::{
    io::{self, Read},
    pin::Pin,
    task::{
        Context,
        Poll::{self, Pending, Ready},
    },
};
/// NOTE: the reader should be set to non-blocking mode otherwise this will block, in other words `.read` should return `io::ErrorKind::WouldBlock`
pub struct ReadFuture<'t, 'b, R: Read> {
    pub reader: &'t mut R,
    pub buf: &'b mut [u8],
}

impl<R: Read> Future for ReadFuture<'_, '_, R> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.reader.read(this.buf) {
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
pub trait AsyncRead {
    #[allow(async_fn_in_trait)]
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}
impl<R: AsyncRead> AsyncRead for &mut R {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return (*self).read(buf).await;
    }
}
