use std::io;

use crate::future::{AsyncBufferReader, AsyncRead, BufferRead};

/// - `None`    → done.
/// - `Some(0)` → no chunk in progress, next read starts a new chunk-size line.
/// - `Some(n)` → n bytes left in the current chunk.
pub struct ChunkedBodyReader<AR: AsyncRead> {
    raw: AsyncBufferReader<AR>,
    remaining: Option<usize>,
}

impl<AR: AsyncRead> ChunkedBodyReader<AR> {
    pub fn new(raw: AsyncBufferReader<AR>) -> Self {
        Self {
            raw,
            remaining: Some(0),
        }
    }

    async fn next_chunk_size(&mut self) -> io::Result<usize> {
        let mut line = Vec::new();
        self.raw
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "failed to read chunk size line"))?;

        let raw_line = String::from_utf8_lossy(&line);
        let hex = raw_line.split(';').next().unwrap_or("").trim();
        usize::from_str_radix(hex, 16).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid chunk size"))
    }

    async fn consume_crlf(&mut self) -> io::Result<()> {
        let mut discard = Vec::new();
        self.raw
            .read_until(b'\n', &mut discard)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "expected CRLF after chunk data"))?;
        Ok(())
    }

    // RFC 9112 §7.1.2
    async fn drain_trailers(&mut self) -> io::Result<()> {
        loop {
            let mut line = Vec::new();
            self.raw
                .read_until(b'\n', &mut line)
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "failed to drain trailers"))?;
            if matches!(line.as_slice(), b"\r\n" | b"\n") || line.is_empty() {
                return Ok(());
            }
        }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut remaining = match self.remaining {
            None => return Ok(0),
            Some(0) => {
                let size = self.next_chunk_size().await?;
                if size == 0 {
                    self.drain_trailers().await?;
                    self.remaining = None;
                    return Ok(0);
                }
                size
            }
            Some(n) => n,
        };

        let to_read = buf.len().min(remaining);
        let n = self.raw.read(&mut buf[..to_read]).await?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed mid-chunk",
            ));
        }

        remaining -= n;
        if remaining == 0 {
            self.consume_crlf().await?;
        }
        self.remaining = Some(remaining);
        Ok(n)
    }

    pub fn into_raw(self) -> AsyncBufferReader<AR> {
        self.raw
    }
}
