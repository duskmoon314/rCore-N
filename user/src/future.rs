use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub struct GetWakerFuture;

impl Future for GetWakerFuture {
    type Output = Waker;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let waker = cx.waker().clone();
        Poll::Ready(waker)
    }
}
