use std::{
    io,
    net::{TcpListener, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};

use crate::http::Connection;

pub struct Server {}

impl Server {
    pub fn new<A: ToSocketAddrs>(name: &str, address: A) -> std::io::Result<()> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true).expect("Cannot set non-blocking");
        println!("[server.{name}] is listening at http://{}", listener.local_addr()?);

        let mut connections: Vec<Connection> = Vec::new();

        loop {
            sleep(Duration::from_secs(1));
            for conn in &mut connections {
                conn.handle_connection();
            }

            let conn_res = listener.accept();
            match conn_res {
                Ok((s, addr)) => connections.push(Connection::new(s, addr)),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => panic!("encountered IO error: {e}"),
            }
        }
    }
}
