use tokio::sync::broadcast;

use crate::error::ErrorSpecialAnA;

pub struct TokenUpdatedPassword {
    pub rx: broadcast::Receiver<String>,
}

impl TokenUpdatedPassword {
    pub async fn password_updated(&mut self) -> Result<String, ErrorSpecialAnA> {
        self.rx
            .recv()
            .await
            .map_err(|err| ErrorSpecialAnA::FetchingToken(err.to_string()))
    }
}

//use pin_project::pin_project;

//#[derive(Clone)]
//#[pin_project]
//pub struct TokenUpdated<F> {
//#[pin]
//pub future: F,
//}

//impl<F> std::future::Future for TokenUpdated<F>
//where
//F: std::future::Future + Send,
//{
//type Output = F::Output;

//fn poll(
//self: std::pin::Pin<&mut Self>,
//cx: &mut std::task::Context<'_>,
//) -> std::task::Poll<Self::Output> {
//let this = self.project();
//this.future.poll(cx)
//}
//}
