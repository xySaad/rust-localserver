use std::{collections::HashMap, net::TcpStream};

pub struct RequestMeta {
    pub method: String,
    pub request_target: String,
    pub protocol: String,
}
pub type RequestHeaders = HashMap<String, Vec<String>>;
pub struct Request {
    pub meta: RequestMeta,
    pub headers: RequestHeaders,
    pub body: TcpStream,
}

impl Request {
    pub fn new(meta: RequestMeta, body: TcpStream) -> Self {
        return Request {
            meta,
            headers: HashMap::new(),
            body,
        };
    }
}
