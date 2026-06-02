use crate::http::{RequestParser, connection::ConnectionState::Parsing};
use std::{net::SocketAddr, net::TcpStream};

enum ConnectionState {
    Parsing(RequestParser),
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

        // println!("{addr:?}");

        let res = match state {
            Parsing(http_request_parser) => http_request_parser.parse(stream),
        };

        // match res {
        //     Ok(()) => (),
        //     Err(e) => println!("{e:?}"),
        // }
    }
}
