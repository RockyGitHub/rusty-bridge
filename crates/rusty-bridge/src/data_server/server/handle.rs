use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::error;

use super::data_events::{ConnectionEvent, DataEvent};
use crate::error::RustyBridgeError;

pub struct DataServerHandle {
    task: Option<JoinHandle<Result<(), RustyBridgeError>>>,
    tx_events: Sender<DataEvent>,
}

impl DataServerHandle {
    pub fn new(
        task: JoinHandle<Result<(), RustyBridgeError>>,
        tx_events: Sender<DataEvent>,
    ) -> DataServerHandle {
        DataServerHandle {
            task: Some(task),
            tx_events,
        }
    }

    pub fn event_connection(&self, connected: bool) {
        let event = DataEvent::ConnectionEvent(ConnectionEvent {
            utc_time: get_time(),
            connected,
        });

        self.send_data(event);
    }

    pub fn event_rx_data(&self, id: u32) {
        let event = DataEvent::NewMsg {
            utc_time: get_time(),
            id,
        };

        self.send_data(event);
    }

    pub fn event_pub_data(&self, id: u32) {
        let event = DataEvent::PubMsg {
            utc_time: get_time(),
            id,
        };

        self.send_data(event);
    }

    pub fn event_pub_ack(&self, id: u32, success: bool) {
        let event = DataEvent::AckMsg {
            utc_time: get_time(),
            success,
            id,
        };

        self.send_data(event);
    }

    fn send_data(&self, event: DataEvent) {
        if let Err(err) = self.tx_events.try_send(event) {
            // If this fails, maybe the server crashed but that shouldn't happen. Maybe it's too busy.
            // TODO - can we self heal in this situation?
            error!("Could not send event to data_server [{:?}]", err);
        }
    }
}

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0))
        .as_millis()
}

impl Drop for DataServerHandle {
    fn drop(&mut self) {
        // If this turns into a Cloneable struct, call Arc::to_inner() to block on the task if it's the final Arc<>
        if let Some(task) = self.task.take() {
            task.abort();
            //match Handle::current().block_on(task) {
            //Ok(_) => (),
            //Err(_) => todo!(),
            //}
        }
    }
}
