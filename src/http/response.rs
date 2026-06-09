use std::io;

use crate::{
    future::{AsyncTcpStream, AsyncWrite},
    http::{Headers, Status},
};

pub struct Response<'t> {
    stream: &'t mut AsyncTcpStream,
}

impl<'t> Response<'t> {
    pub fn new(stream: &'t mut AsyncTcpStream) -> Self {
        Self { stream }
    }
    pub async fn status(self, status: Status) -> HeadersWriter<'t, AsyncTcpStream> {
        let status_line = format!("HTTP/1.1 {status} {status:?}\r\n");
        _ = self.stream.write(status_line.as_bytes()).await;
        return HeadersWriter { writer: self.stream };
    }
}

pub struct HeadersWriter<'t, W: AsyncWrite> {
    writer: &'t mut W,
}

impl<'t, W: AsyncWrite> HeadersWriter<'t, W> {
    pub async fn headers(self, headers: Headers) -> BodyWriter<'t, W> {
        for (k, values) in headers {
            for v in values {
                let header_line = format!("{k}: {v}\r\n");
                _ = self.writer.write(header_line.as_bytes()).await;
            }
        }
        _ = self.writer.write("\r\n".as_bytes()).await;

        return BodyWriter { writer: self.writer };
    }
}

pub struct BodyWriter<'t, W: AsyncWrite> {
    writer: &'t mut W,
}

impl<'t, W: AsyncWrite> AsyncWrite for BodyWriter<'t, W> {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.writer.write(buf).await;
    }
}
