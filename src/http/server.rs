use std::{
    io,
    net::{TcpListener, ToSocketAddrs},
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll::{Pending, Ready},
        Wake, Waker,
    },
    thread::sleep,
    time::Duration,
};

use crate::http::Connection;

pub struct ExampleWaker {}
impl Wake for ExampleWaker {
    fn wake(self: Arc<Self>) {
        println!("waking...")
    }
}

pub struct Server {}

impl Server {
    pub fn new<A: ToSocketAddrs>(name: &str, address: A) -> std::io::Result<()> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true).expect("Cannot set non-blocking");
        println!("[server.{name}] is listening at http://{}", listener.local_addr()?);

        let mut connections: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

        loop {
            sleep(Duration::from_secs(1));
            connections.retain_mut(|fut| {
                let arc = Arc::new(ExampleWaker {});
                let waker = &Waker::from(arc);
                let cx = &mut Context::from_waker(waker);
                match fut.as_mut().poll(cx) {
                    Ready(_) => false,
                    Pending => true,
                }
            });

            let conn_res = listener.accept();
            match conn_res {
                Ok((s, addr)) => {
                    let conn = Connection::new(s, addr)?;
                    let fut = Box::pin(async {
                        let mut conn = conn;
                        conn.handle_connection().await;
                    });
                    connections.push(fut);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => panic!("encountered IO error: {e}"),
            }
        }
    }
}
