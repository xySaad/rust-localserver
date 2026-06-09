use std::collections::HashMap;

use crate::{future::AsyncTcpStream, http::Response};

pub struct RequestLine {
    pub method: String,
    pub request_target: String,
    pub protocol: String,
}

pub type Headers = HashMap<String, Vec<String>>;
pub struct Request<'t> {
    pub meta: RequestLine,
    pub headers: Headers,
    body: &'t mut AsyncTcpStream,
}

impl<'t> Request<'t> {
    pub fn new(meta: RequestLine, body: &'t mut AsyncTcpStream, headers: Headers) -> Self {
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
    pub fn response(self) -> Response<'t> {
        Response::new(self.body)
    }
}
