use std::{collections::HashMap, io};

use crate::{
    future::{AsyncBufferReader, AsyncRead, AsyncWrite},
    http::{Response, body_reader::BodyReader},
};

pub struct RequestLine {
    pub protocol: String,
    pub method: String,
    pub path: String,
    pub query: String,
}

pub type Headers = HashMap<String, Vec<String>>;

pub struct Request<AR: AsyncRead> {
    pub meta: RequestLine,
    pub headers: Headers,
    body_reader: BodyReader<AR>,
}

impl<AR: AsyncRead> Request<AR> {
    pub fn new(meta: RequestLine, raw: AsyncBufferReader<AR>, headers: Headers) -> Self {
        let body_reader = BodyReader::from_headers(&headers, raw);
        Request {
            meta,
            headers,
            body_reader,
        }
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key)?.first().map(String::as_str)
    }

    /// Reads the next chunk of body bytes into `buf`, per whichever framing (RFC 9112 §6).
    /// Returns `Ok(0)` once the body is exhausted.
    pub async fn read_body(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.body_reader.read(buf).await
    }

    /// reads the whole body into a `Vec<u8>` and returns it
    pub async fn body(&mut self) -> io::Result<Vec<u8>> {
        let mut body = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let n = self.read_body(&mut chunk).await?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
        Ok(body)
    }
}

impl<ARW: AsyncRead + AsyncWrite> Request<ARW> {
    pub fn response(self) -> Response<ARW> {
        Response::new(self.body_reader.into_raw().take_reader())
    }
    pub async fn write_raw_response(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.body_reader.into_raw_mut().as_mut_reader().write(buf).await;
    }
}
