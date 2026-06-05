use std::{
    io,
    net::{SocketAddr, TcpStream},
};

use crate::{future::AsyncTcpStream, http::request_parser::RequestParser};

pub struct Connection {
    stream: AsyncTcpStream,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> io::Result<Self> {
        let conn = Connection {
            stream: AsyncTcpStream::from(stream)?,
            addr,
        };

        return Ok(conn);
    }

    pub async fn handle_connection(self: &mut Self) {
        let mut parser = RequestParser::from(&mut self.stream);
        match parser.parse().await {
            Ok((start_line, headers)) => {
                println!("method: {}", start_line.method);
                println!("headers: {headers:?}");
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
}
