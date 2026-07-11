use std::io;

use crate::{
    future::AsyncWrite,
    http::{Headers, Status},
};

pub struct Response<WR: AsyncWrite> {
    writer: WR,
}

impl<'t, WR: AsyncWrite> Response<WR> {
    pub fn new(writer: WR) -> Self {
        Self { writer }
    }
    pub async fn status(mut self, status: Status) -> HeadersWriter<WR> {
        let status_line = format!("HTTP/1.1 {status} {status:?}\r\n");
        _ = self.writer.write(status_line.as_bytes()).await;
        return HeadersWriter { writer: self.writer };
    }
}

pub struct HeadersWriter<WR: AsyncWrite> {
    writer: WR,
}

impl<'t, W: AsyncWrite> HeadersWriter<W> {
    pub async fn headers(mut self, headers: Headers) -> BodyWriter<W> {
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

pub struct BodyWriter<W: AsyncWrite> {
    writer: W,
}

impl<W: AsyncWrite> AsyncWrite for BodyWriter<W> {
    async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.writer.write(buf).await;
    }
}
