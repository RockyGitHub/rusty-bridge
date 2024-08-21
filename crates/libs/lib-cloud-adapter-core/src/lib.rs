use std::future::Future;

use async_trait::async_trait;
use data_source_core::MsgBusData;
use pin_project::pin_project;
use thiserror::Error;
use tokio::sync::watch;

pub type DeliveryToken = Box<dyn Future<Output = Result<(), DeliveryError>>>;
// Not possible to alias impl yet, but it's being worked on https://github.com/rust-lang/rust/issues/63063
//pub type TokenConnection = TokenConnection<impl Future<Output = Result<(), ConnectionError>>, ConnectionError>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("initialization {0}")]
    Initialization(String),
}

#[derive(Debug)]
pub struct DeliveryError {
    pub msg_id: u32,
    pub reason: String,
}
//pub enum DeliveryError {
//Failure(u32, String), // (msg_id, reason)
//}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("connection success")]
    Success,
    #[error("connection failed due to: [{0}]")]
    Failure(String),
}

#[derive(Debug, Error)]
pub enum ConnectionLost {
    #[error("manual disconnect")]
    ManualDisconnect,
    #[error("timed out")]
    Timeout,
    #[error("unknown")]
    Uncategorized(String),
}

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub custom: Option<String>,
}

//#[enum_dispatch]
pub trait CloudAdapterTrait {
    // the 'static is important when passing this to a tokio task
    /// Publish a message through the cloud adapter.
    /// * Returns a token to await for the acknowledgement of delivery to the receiving side
    fn publish(&mut self, msg: MsgBusData) -> impl TokenDelivery + Send + 'static;
    /// Returns a token to await the result of the connection attempt
    /// * On success, the Ok(rx) returns a channel that will receive a notice if connection is lost
    async fn connect(
        &mut self,
    ) -> Result<
        TokenConnection<
            impl Future<Output = Result<watch::Receiver<ConnectionLost>, ConnectionError>>
                + Send
                + 'static,
        >,
        ConnectionError,
    >;
    fn disconnect(
        &mut self,
    ) -> Result<impl Future<Output = Result<(), ConnectionError>> + Send + 'static, ConnectionError>;
}

#[async_trait]
pub trait TokenDelivery {
    //fn wait_for_ack(self) -> impl std::future::Future<Output = Result<(), DeliveryError>> + Send;
    async fn wait_for_ack(self) -> Result<u32, DeliveryError>;
}

#[pin_project]
pub struct TokenConnection<F> {
    #[pin]
    pub future: F,
}

#[pin_project::pin_project]
pub struct TokenDisconnect<F> {
    #[pin]
    pub future: F,
}

impl<F> std::future::Future for TokenConnection<F>
where
    F: std::future::Future + Send,
{
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx)
    }
}

impl<F> std::future::Future for TokenDisconnect<F>
where
    F: std::future::Future + Send,
{
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx)
    }
}

//#[async_trait]
//pub trait TokenConnection {
//async fn wait_for_connect(self) -> Result<(), ConnectionError>;
//}
