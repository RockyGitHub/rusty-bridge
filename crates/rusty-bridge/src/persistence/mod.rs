use data_source_core::MsgBusData;
use msg_persistence::MsgPersistence;
use tokio::{
    pin, select,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tracing::{debug, info, trace, warn};

const PUBLISH_THREAD_NAME: &str = "publish on connect";
const THROTTLE_RATE_MS: u64 = 5; // 200 per second

// TODO - this can be abstracted for different persistence types, but that's probably not an avenue I'll take anytime soon
pub fn init_persistence() -> MsgPersistence {
    MsgPersistence::new()
}

/// Starts the thread that listens for connection notices.
/// If a connection status of true comes through, the database will be iterated through and each message sent through the publishing channel
pub fn start_persistence_publish_thread(
    persistence: MsgPersistence,
    mut rx_conn_status: broadcast::Receiver<bool>,
    tx_publish: mpsc::Sender<MsgBusData>,
) -> JoinHandle<()> {
    tokio::task::spawn(
        async move { publish_on_connect(persistence, rx_conn_status, tx_publish).await },
    )
}

async fn publish_on_connect(
    persistence: MsgPersistence,
    mut rx_conn_status: broadcast::Receiver<bool>,
    tx_publish: mpsc::Sender<MsgBusData>,
) {
    debug!("Starting '{}' thread", PUBLISH_THREAD_NAME);
    loop {
        // TODO - Every x minutes, check if messages exist in the db. If they do, and we are connected, send them?
        // alternatively, maybe subscribe for the event of something being persisted, if it occurs, unsubscribe and send all?
        // but if something is persisted, then the internet is possibly out

        // Hoist the futures
        pin! {
            let conn_status = rx_conn_status.recv();
        }

        select! {
            mailbox = conn_status => {
                match mailbox {
                    Ok(status) =>
                        if true == status {
                            info!("Beginning to send [{}] msgs from persistence!", 1);
                            // Fake loop until this is implemented
                            for _ in 0..1 {
                                let data = MsgBusData {payload:"data from persistence".as_bytes().to_vec() ,retry_count:1, id: 0 };
                                trace!("Sending persisted message to publishing");
                                let _ = tx_publish.send(data).await;
                                tokio::time::sleep(tokio::time::Duration::from_millis(THROTTLE_RATE_MS)).await;
                            }
                        } else {
                            // This is mostly here for debugging purposes, see if the thread is alive, etc
                            trace!("Persistence, publish on connect thread rxd false connection status. Nothing to do..")
                        },
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => break,
                        broadcast::error::RecvError::Lagged(skipped_count) => warn!("Lag detected in '{}', missed status msg count: [{}]", PUBLISH_THREAD_NAME, skipped_count),
                    },
                }
            }
        }

        info!("Exiting persistence '{}' thread", PUBLISH_THREAD_NAME);
    }
}
