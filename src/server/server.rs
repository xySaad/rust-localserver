use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    // ...
}

pub struct Server {}

impl Server {
    pub fn new(name: &str, address: &str) -> std::io::Result<()> {
        let listener = TcpListener::bind(address)?;
        println!("[server.{name}] is listening at {address}");

        for stream in listener.incoming() {
            handle_client(stream?);
        }
        Ok(())
    }
}
