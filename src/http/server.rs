use std::{
    cell::{BorrowMutError, RefCell},
    net::{TcpListener, ToSocketAddrs},
    rc::Rc,
};

use crate::{
    future::{AsyncTcpListener, Pool, Task},
    http::{Connection, Request},
};

pub struct Server<'t> {
    pub listener: AsyncTcpListener,
    pool: Rc<RefCell<Pool<'t>>>,
}

impl<'t> Server<'t> {
    //Creates a new TcpListener which will be bound to the specified address.
    //And sets nonblocking mode to true
    pub fn bind<A: ToSocketAddrs>(address: A) -> std::io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        return Ok(Self {
            pool: Rc::new(RefCell::new(Pool::new())),
            listener: AsyncTcpListener::from(listener)?,
        });
    }

    async fn accept<H: AsyncFn(Request<'_>)>(&mut self, handler: &'t H) -> Result<(), BorrowMutError> {
        match self.listener.accept().await {
            Ok((s, addr)) => {
                let conn = Connection::new(s, addr);
                println!("connection arrived");
                let mut mx = self.pool.try_borrow_mut()?;
                println!("adding connection task");
                mx.add_task(Task::new(async move {
                    let mut conn = conn;
                    conn.handle_connection(handler).await;
                }));
            }
            Err(e) => println!("error accepting connection {e}"),
        }

        return Ok(());
    }

    pub async fn serve<H: AsyncFn(Request<'_>)>(self, handler: &'t H) {
        self.build_executor(handler).await_all().await;
    }

    pub fn serve_and_block<H: AsyncFn(Request<'_>)>(self, handler: &'t H) -> () {
        self.build_executor(handler).block();
    }

    pub fn build_executor<H: AsyncFn(Request<'_>)>(self, handler: &'t H) -> Pool<'t> {
        let accept_pool = Rc::clone(&self.pool);
        let mut this = self;
        let task = Task::new(async move {
            loop {
                if let Err(er) = this.accept(handler).await {
                    println!("error borrowing pool clone {er}")
                }
            }
        });

        let await_task = Task::new(async move {
            Pool::ref_await_all(accept_pool.as_ref())
                .await
                .unwrap_or_else(|e| println!("error borrowing pool {e}"));
        });

        let mut executor = Pool::new();
        executor.add_task(task);
        executor.add_task(await_task);
        return executor;
    }
}
