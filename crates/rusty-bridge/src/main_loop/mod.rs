use cloud_adapter_core::{
    CloudAdapterTrait, ConnectionError, ConnectionLost, DeliveryError, TokenDelivery,
};
use data_source_core::{DataSourceInterface, MsgBusData, RxData};
use tokio::{
    select, spawn,
    sync::{
        mpsc::{self, error::TrySendError, Receiver, Sender},
        watch,
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

use crate::data_server::DataServerHandle;

// TWO states of operation.  Regular and Persistence
// Regular mode: -only enters after exiting persistence mode
//  Receive msg on channel from msg_bus
//  Extract and transform the message
//  Publish the message
//  Spawn a task that will await the token
//  On Ack, do nothing
//  On NACK, add msg to persistence
// Persistence: -enters on connection=true
//  Spawn task that will iterate through db and publish messages
//      pass it a tx channel, it be used as a special publishing channel so the adapter doesn't need to be passed to the task.
//      It should probably be dynamically throttled too, so maybe pass it an rx channel so its speed can be tuned dynamically
//      when done, return to regular mode
//  Receive msg on channel from msg_bus
//  Extract and transform the message
//  add msg to persistence
//

// Also:
// Priority channel - this channel has minimal activity and is reserved for adapter choice. For example, heartbeats and command received
//

pub async fn main_loop(
    metrics_events: DataServerHandle,
    mut adapter: impl CloudAdapterTrait + Send,
    mut rx_msg: RxData,
    shutdown_token: CancellationToken,
) {
    let mut ack_tasks = create_ack_task_set(shutdown_token.clone());
    let mut connect_tasks = create_connect_task_set(shutdown_token.clone());
    let (tx_conn_status, mut rx_conn_status) = mpsc::channel(10);

    // Ready to start, begin with attempting a connection to the cloud
    let token = adapter.connect().await.unwrap();
    connect_tasks.spawn(async move { token.await });

    let mut connected = false;

    let exit_reason = loop {
        select! {
            // Receive messages to publish
            mailbox = rx_msg.recv() => {
                match mailbox {
                    Some(msg) => {
                        if connected {
                            trace!("Publishing [{:?}]", msg);
                            debug!("Publishing [{}]", msg.id);
                            metrics_events.event_pub_data(msg.id);
                            let token = adapter.publish(msg);
                            ack_tasks.spawn(async move {
                                handle_token(token).await
                            });
                        } else {
                            debug!("TODO - Persisting msg [{}] due to no connectivity", msg.id)
                        }
                    },
                    None => break "tx dropped",
                }
            },
            // Handle what to do when a published message's token resolves
            // TODO - benchmark if it's faster to pass the token through a channel and handle this joinset in its own task
            mailbox_task = ack_tasks.join_next() => {
                match mailbox_task {
                    Some(join_result) => match join_result {
                        Ok(ack_result) => {
                            match ack_result {
                                Ok(msg_id) => {
                                    debug!("Msg Ack. Msg Id: [{}]", msg_id);
                                    metrics_events.event_pub_ack(msg_id, true);
                                    // TODO - remove message from persistence
                                    trace!("todo: removing Msg Id [{}] from persistence", msg_id);
                                },
                                Err(err) => {
                                    warn!("No Ack received for msg: [{}], reason: [{}]", err.msg_id, err.reason);
                                    metrics_events.event_pub_ack(err.msg_id, false);
                                    // TODO - persistence
                                    trace!("todo: adding to msg id [{}] to persistence", err.msg_id);
                                },
                            }
                        },
                        Err(err) => {
                            warn!("Join error on ack result. [{}]", err);
                        },
                    },
                    None => break "ack_tasks empty",
                }
            },
            // Receive a connectivity status update
            mailbox = rx_conn_status.recv() => {
                match mailbox {
                    Some(conn_status) => {
                        metrics_events.event_connection(conn_status);
                        if conn_status == false {
                            connected = false;
                            // The if statement below safeguards against creating additional connection tasks when it is already occuring
                            // I don't know if this situation could ever occur, but this is just incase
                            if connect_tasks.len() == 1 { // there's 1 dummy task, so check for 1
                                let token = adapter.connect().await.unwrap();
                                connect_tasks.spawn(async move {
                                    token.await
                                });
                            } else {
                                error!("Multiple connection tasks attempted, this msg is to indiciate it is occuring but shouldn't be")
                            }
                        } else {
                        }
                    },
                    None => break "rx_conn_status closed",
                }
            },
            // What to do when a connection is succesful or not
            mailbox_conn_task = connect_tasks.join_next() => {
                match mailbox_conn_task {
                    Some(join_result) => match join_result {
                        Ok(connection_result) => match connection_result {
                            Ok(mut rx_conn_lost) => {
                                connected = true;
                                info!("Connection attempt success");
                                // Burn whatever may have last been posted in rx_conn_lost
                                let _ = rx_conn_lost.borrow_and_update();
                                tx_conn_status.try_send(true).unwrap();
                                // A token indicating connection lost is passed in.
                                // Now we spawn a task to notify this loop of a connection loss
                                let tx_conn_status_clone = tx_conn_status.clone();
                                spawn(async move {
                                    match rx_conn_lost.changed().await {
                                        Ok(_) => {
                                            let reason = rx_conn_lost.borrow_and_update().to_string(); // This gets weird without the `.to_string()`
                                            warn!("Connection lost: [{}]", reason);
                                        },
                                        Err(err) => debug!("Connection possibly lost, sender was dropped: [{}]", err), // debug until I pass the cancel token to this task and priority the token in select!
                                    }
                                    if let Err(err) = tx_conn_status_clone.try_send(false) {
                                        debug!("Could not update connection status, this may cause critical issues. [{}]", err) // debug until I pass the cancel token to this task and priority the token in select!
                                    }
                                });
                            },
                            Err(err) => {
                                // Connection attempt failed, try it again
                                warn!("Connection attempt failed. [{}]", err);
                                let token = adapter.connect().await.unwrap();
                                connect_tasks.spawn(async move {
                                    token.await
                                });
                            }
                        },
                        Err(err) => {
                            error!("Join failed on connection task [{}]", err);
                            let token = adapter.connect().await.unwrap();
                            connect_tasks.spawn(async move {
                                token.await
                            });
                        },
                    },
                    None => break "connect tasks are empty",
                }
            },
            _ = shutdown_token.cancelled() => break "shutdown token was cancelled"
        }
    };
    info!("Exiting main_loop [{}]", exit_reason);

    if let Ok(token) = adapter.disconnect() {
        let _ = token.await;
    }
}

async fn handle_token(token: impl TokenDelivery) -> Result<u32, DeliveryError> {
    token.wait_for_ack().await
}

fn create_ack_task_set(shutdown: CancellationToken) -> JoinSet<Result<u32, DeliveryError>> {
    let mut tasks = JoinSet::new();

    // Spawn an indefinite task so that .join_next() doesn't return None
    tasks.spawn(async {
        shutdown.cancelled_owned().await;
        Ok(0)
    });
    tasks
}

// Wrap a join set that handles joining all tasks and then always spawning a dummy task?
fn abort_ack_set(ack_set: &mut JoinSet<()>) {
    ack_set.shutdown();
    ack_set.spawn(async {
        // replace this sleep with waiting on a shutdown.changed().await channel
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await
    });
}

fn create_connect_task_set(
    shutdown: CancellationToken,
) -> JoinSet<Result<watch::Receiver<ConnectionLost>, ConnectionError>> {
    let mut tasks = JoinSet::new();

    // Spawn an indefinite task so that .join_next() doesn't return None
    tasks.spawn(async {
        shutdown.cancelled_owned().await;
        Err(ConnectionError::Failure(
            "connection tasks shutting down".to_string(),
        ))
    });
    tasks
}
