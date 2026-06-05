use std::{collections::HashMap, net::TcpStream};

pub struct RequestLine {
    pub method: String,
    pub request_target: String,
    pub protocol: String,
}

pub type RequestHeaders = HashMap<String, Vec<String>>;
pub struct Request {
    pub meta: RequestLine,
    pub headers: RequestHeaders,
    pub body: TcpStream,
}

impl Request {
    pub fn new(meta: RequestLine, body: TcpStream, headers: RequestHeaders) -> Self {
        return Request { meta, headers, body };
    }
}
