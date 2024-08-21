use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::runtime::Handle;
use tokio::select;
use tokio::sync::mpsc::{self};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::data_server::server::data_service::DataService;
use crate::error::RustyBridgeError;
use crate::initialize::InitError;

use super::data_events::DataEvent;
use super::handle::DataServerHandle;

const SERVER_IP: &str = "127.0.0.1";
const SERVER_PORT: u16 = 9000;

pub async fn init_data_server(shutdown: CancellationToken) -> Result<DataServerHandle, InitError> {
    info!(
        "Starting data server at [{}]",
        format!("{}:{}", SERVER_IP, SERVER_PORT)
    );

    let ip = IpAddr::from_str(SERVER_IP).map_err(|err| {
        InitError::DataServer(format!("data_server ip addr parse failed: [{}]", err))
    })?;
    let addr = SocketAddr::new(ip, SERVER_PORT);
    let listener = TcpListener::bind(addr).await.map_err(|err| {
        InitError::DataServer(format!(
            "data_server failed to bind tcp listener: [{}]",
            err
        ))
    })?;

    let (tx_events, rx_events) = tokio::sync::mpsc::channel(10);
    let handle = Handle::current().spawn(server(shutdown, listener, rx_events));

    let data_server_handle = DataServerHandle::new(handle, tx_events);

    Ok(data_server_handle)
}

async fn server(
    shutdown: CancellationToken,
    listener: TcpListener,
    mut rx_event: mpsc::Receiver<DataEvent>,
) -> Result<(), RustyBridgeError> {
    let mut stream_set = JoinSet::new();

    let mut data_service = DataService::new();
    loop {
        select! {
            // Listen and accept new client connections
            new_client = listener.accept() => {
                let Ok((stream, _)) = new_client else {
                    continue;
                };
                let io = TokioIo::new(stream);
                let data_service_clone = data_service.clone();
                stream_set.spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, data_service_clone)
                        .await {
                            error!("Could not serve connection: [{}]", err);
                        }
                });
            },
            // Handle updating the database with a new event
            mailbox = rx_event.recv() => {
                match mailbox {
                    Some(event) => data_service.handle_event(event),
                    None => break,
                }
            }
            // Shutdown if the token gets canceled
            _ = shutdown.cancelled() => break

        }
    }

    info!("Exiting data_server");
    Ok(())
}
