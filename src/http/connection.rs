use crate::http::{
    ParsedRequest, RequestParser,
    connection::ConnectionState::{Parsing, Ready},
};
use std::{net::SocketAddr, net::TcpStream};

enum ConnectionState {
    Parsing(RequestParser),
    Ready(ParsedRequest),
}

pub struct Connection {
    stream: TcpStream,
    state: ConnectionState,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        stream.set_nonblocking(true).expect("set_nonblocking call failed");

        return Connection {
            state: Parsing(RequestParser::new()),
            stream,
            addr,
        };
    }

    pub fn handle_connection(self: &mut Self) {
        let Self { state, stream, .. } = self;

        match state {
            Parsing(http_request_parser) => {
                let request = http_request_parser.parse(stream);
                match request {
                    Ok(None) => return,
                    Err(e) => println!("{e:?}"),
                    Ok(Some(res)) => self.state = Ready(res),
                }
            }
            Ready(request) => todo!("Run handler"),
        };
    }
}
