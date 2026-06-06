use std::collections::HashMap;

use crate::future::AsyncTcpStream;

pub struct RequestLine {
    pub method: String,
    pub request_target: String,
    pub protocol: String,
}

pub type RequestHeaders = HashMap<String, Vec<String>>;
pub struct Request<'t> {
    pub meta: RequestLine,
    pub headers: RequestHeaders,
    pub body: &'t mut AsyncTcpStream,
}

impl<'t> Request<'t> {
    pub fn new(meta: RequestLine, body: &'t mut AsyncTcpStream, headers: RequestHeaders) -> Self {
        return Request { meta, headers, body };
    }
}
