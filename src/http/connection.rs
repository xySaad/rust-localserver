use std::net::SocketAddr;

use crate::{future::AsyncTcpStream, http::request_parser::RequestParser};

pub struct Connection {
    stream: AsyncTcpStream,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(stream: AsyncTcpStream, addr: SocketAddr) -> Self {
        return Self { stream, addr };
    }

    pub async fn handle_connection(self: &mut Self) {
        let mut parser = RequestParser::from(&mut self.stream);
        println!("parsing connection");
        match parser.parse().await {
            Ok((start_line, headers)) => {
                println!("method: {}", start_line.method);
                println!("headers: {headers:?}");
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
}
