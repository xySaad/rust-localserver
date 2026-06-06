use std::net::SocketAddr;

use crate::{
    future::AsyncTcpStream,
    http::{Request, request_parser::RequestParser},
};

pub struct Connection {
    stream: AsyncTcpStream,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(stream: AsyncTcpStream, addr: SocketAddr) -> Self {
        return Self { stream, addr };
    }

    pub async fn handle_connection<F: AsyncFn(Request<'_>)>(self: &mut Self, handler: F) {
        let mut parser = RequestParser::from(&mut self.stream);
        println!("parsing connection");
        match parser.parse().await {
            Ok((start_line, headers)) => {
                let req = Request::new(start_line, &mut self.stream, headers);
                handler(req).await;
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
}
