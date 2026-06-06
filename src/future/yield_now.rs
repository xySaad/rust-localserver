use std::{
    pin::Pin,
    task::{
        Context,
        Poll::{self, Pending, Ready},
    },
};

// A minimal future that yields once then resolves,
// letting other tasks run in between
pub struct YieldNow(pub bool);
impl Future for YieldNow {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if self.0 {
            Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Pending
        }
    }
}
