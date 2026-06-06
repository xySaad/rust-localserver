use std::{
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

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}
impl Task {
    pub fn new<F: Future<Output = ()> + 'static>(future: F) -> Self {
        return Self {
            future: Box::pin(future),
        };
    }
}

pub struct Pool {
    pending: Vec<Task>,
    awake: Vec<Task>,
}

pub struct NoOpWaker {}

impl NoOpWaker {}
impl Wake for NoOpWaker {
    fn wake(self: Arc<Self>) {
        println!("waking...")
    }
}

impl Pool {
    pub fn new() -> Self {
        return Self {
            pending: vec![],
            awake: vec![],
        };
    }

    pub fn add_task(&mut self, task: Task) {
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
    }

    pub fn block(&mut self) {
        loop {
            self.poll_once();
            sleep(Duration::from_secs(1));
        }
    }

    pub async fn await_all(&mut self) {
        loop {
            self.poll_once();
            YieldNow(false).await;
            sleep(Duration::from_secs(1));
        }
    }
}
