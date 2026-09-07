use crate::future::{AsyncBufferReader, AsyncRead};
use crate::http::{ChunkedBodyReader, Headers, SizedBodyReader};
use std::io;

pub enum BodyReader<AR: AsyncRead> {
    Sized(SizedBodyReader<AR>),
    Chunked(ChunkedBodyReader<AR>),
    Empty(AsyncBufferReader<AR>),
    Rejected(AsyncBufferReader<AR>, io::ErrorKind, String),
}

impl<AR: AsyncRead> BodyReader<AR> {
    // RFC 9112 §6.3: Transfer-Encoding takes precedence over Content-Length.
    pub fn from_headers(headers: &Headers, raw: AsyncBufferReader<AR>) -> Self {
        // RFC 9112 §6.1: collect all Transfer-Encoding values, flatten
        // comma-separated lists, and inspect the last coding.
        if let Some(te_values) = headers.get("Transfer-Encoding") {
            let last_coding = te_values
                .iter()
                .flat_map(|v| v.split(','))
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .last();

            return match last_coding.as_deref() {
                Some("chunked") => BodyReader::Chunked(ChunkedBodyReader::new(raw)),
                Some(other) => BodyReader::Rejected(
                    raw,
                    io::ErrorKind::Unsupported,
                    format!("unsupported transfer-encoding: {other}"),
                ),
                None => BodyReader::Rejected(
                    raw,
                    io::ErrorKind::InvalidData,
                    "empty Transfer-Encoding header".to_string(),
                ),
            };
        }

        // RFC 9112 §6.3: multiple Content-Length fields must be rejected.
        let cl_values = headers.get("Content-Length");
        if let Some(values) = cl_values {
            if values.len() > 1 {
                return BodyReader::Rejected(
                    raw,
                    io::ErrorKind::InvalidData,
                    "multiple Content-Length headers".to_string(),
                );
            }
            match values.first().and_then(|v| v.parse::<usize>().ok()) {
                Some(0) | None => return BodyReader::Empty(raw),
                Some(len) => return BodyReader::Sized(SizedBodyReader::new(raw, len)),
            }
        }

        BodyReader::Empty(raw)
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            BodyReader::Sized(r) => r.read(buf).await,
            BodyReader::Chunked(r) => r.read(buf).await,
            BodyReader::Empty(_) => Ok(0),
            BodyReader::Rejected(_, kind, msg) => Err(io::Error::new(*kind, msg.clone())),
        }
    }

    pub fn into_raw(self) -> AsyncBufferReader<AR> {
        match self {
            BodyReader::Sized(r) => r.into_raw(),
            BodyReader::Chunked(r) => r.into_raw(),
            BodyReader::Empty(raw) => raw,
            BodyReader::Rejected(raw, _, _) => raw,
        }
    }

    pub fn into_raw_mut(&mut self) -> &mut AsyncBufferReader<AR> {
        match self {
            BodyReader::Sized(r) => r.into_raw_mut(),
            BodyReader::Chunked(r) => r.into_raw_mut(),
            BodyReader::Empty(raw) => raw,
            BodyReader::Rejected(raw, _, _) => raw,
        }
    }
}
