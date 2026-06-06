use std::{
    cell::{BorrowMutError, RefCell},
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

use crate::future::YieldNow;

pub struct Task<'t> {
    future: Pin<Box<dyn Future<Output = ()> + 't>>,
}
impl<'t> Task<'t> {
    pub fn new<F: Future<Output = ()> + 't>(future: F) -> Self {
        return Self {
            future: Box::pin(future),
        };
    }
}

pub struct Pool<'t> {
    pending: Vec<Task<'t>>,
    awake: Vec<Task<'t>>,
}

pub struct NoOpWaker {}

impl NoOpWaker {}
impl Wake for NoOpWaker {
    fn wake(self: Arc<Self>) {
        // println!("waking...")
    }
}

impl<'t> Pool<'t> {
    pub fn new() -> Self {
        return Self {
            pending: vec![],
            awake: vec![],
        };
    }

    pub fn add_task(&mut self, task: Task<'t>) {
        self.awake.push(task);
    }

    pub fn poll_once(&mut self) {
        self.awake.retain_mut(|task| {
            let arc = Arc::new(NoOpWaker {});
            let waker = Waker::from(arc);
            let cx = &mut Context::from_waker(&waker);
            match task.future.as_mut().poll(cx) {
                Ready(_) => false,
                Pending => true,
            }
        });
        sleep(Duration::from_nanos(1));
    }

    pub fn block(&mut self) {
        loop {
            self.poll_once();
        }
    }

    pub async fn await_all(&mut self) {
        loop {
            self.poll_once();
            YieldNow(false).await;
        }
    }

    pub async fn ref_await_all(this: &RefCell<Self>) -> Result<(), BorrowMutError> {
        loop {
            this.try_borrow_mut()?.poll_once();
            YieldNow(false).await;
        }
    }
}
