use std::{collections::HashMap, io};

use crate::{
    future::{AsyncBufferReader, AsyncRead, AsyncWrite},
    http::Response,
};

pub struct RequestLine {
    pub method: String,
    pub request_target: String,
    pub protocol: String,
}

pub type Headers = HashMap<String, Vec<String>>;
pub struct Request<AR: AsyncRead> {
    pub meta: RequestLine,
    pub headers: Headers,
    body: AsyncBufferReader<AR>,
}

impl<'t, AR: AsyncRead> Request<AR> {
    pub fn new(meta: RequestLine, body: AsyncBufferReader<AR>, headers: Headers) -> Self {
        return Request { meta, headers, body };
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        if let Some(headers) = self.headers.get(key) {
            if let Some(header) = headers.get(0) {
                return Some(header);
            }
        }

        return None;
    }

    pub async fn body(&mut self) -> io::Result<Vec<u8>> {
        let len = self
            .header("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut buf = vec![0u8; len];
        let mut total = 0;
        while total < len {
            let n = self.body.read(&mut buf[total..]).await?;
            if n == 0 {
                break;
            }
            total += n;
        }
        buf.truncate(total);

        Ok(buf)
    }
}

impl<'t, ARW: AsyncRead + AsyncWrite> Request<ARW> {
    pub fn response(self) -> Response<ARW> {
        Response::new(self.body.take_reader())
    }
}
