use pin_project_lite::pin_project;
use std::pin::Pin;

//############################
// no unsafe
pin_project! {
    struct Foo<F> {
        #[pin]
        future: F,
    }
}

impl<F> std::future::Future for Foo<F>
where
    F: std::future::Future,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx)
    }
}
// no unsafe
// ####################################################
// ####################################################
// ####################################################
// ####################################################
// ####################################################
// ####################################################

//unsafe
// why is this unsafe? because you need to get a mutable reference to the member to construct the Pin<&mut F>
// and getting a mutable reference to a pinned object is unsafe ----- HomelikeBrick42
// "technically, it is more efficient (benchmark needed)"
use std::pin::Pin;

struct Foo<F> {
    future: F,
}

impl<F> std::future::Future for Foo<F>
where
    F: std::future::Future,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|this| &mut this.future) }.poll(cx)
    }
}
//unsafe

// This maybe works too, maybe but only allows polling the future once
struct Foo<F> {
    future: F,
}

impl<F> std::future::Future for Foo<F>
where
    F: std::future::Future + std::marker::Unpin,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.get_mut().future).poll(cx)
    }
}
