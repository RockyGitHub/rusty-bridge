use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use eframe::web_sys::console::info;
use egui::NumExt;
use log::{info, warn};
use wasm_timer::Instant;

use crate::{data_collection::msg_events::DataEvent, Error};

use self::{conn_events::ConnEventData, msg_events::MsgEventData};

pub mod conn_events;
pub mod msg_events;

pub struct RustyBridgeData {
    pub connection_events: Vec<ConnEventData>,
    pub msg_events: Vec<MsgEventData>,
    // msg events
    tx_msg_events: Sender<Vec<MsgEventData>>,
    rx_msg_events: Receiver<Vec<MsgEventData>>,
    // conn events
    tx_conn_events: Sender<Vec<ConnEventData>>,
    rx_conn_events: Receiver<Vec<ConnEventData>>,

    pub activity_strings: f64,
    pub time_since_last_update: Instant,
}

impl RustyBridgeData {
    pub fn new() -> RustyBridgeData {
        let (tx, rx) = mpsc::channel();
        let (tx_conn, rx_conn) = mpsc::channel();
        RustyBridgeData {
            connection_events: Vec::new(),
            msg_events: Vec::new(),
            tx_msg_events: tx,
            rx_msg_events: rx,
            tx_conn_events: tx_conn,
            rx_conn_events: rx_conn,
            activity_strings: 0.0,
            time_since_last_update: Instant::now(),
        }
    }

    pub fn try_recv(&mut self) -> () {
        match self.rx_msg_events.try_recv() {
            Ok(data) => {
                self.msg_events = data;
                self.activity_strings = self.time_since_last_update.elapsed().as_millis() as f64;
                self.time_since_last_update = Instant::now();
            }
            Err(_err) => {
                self.activity_strings -= 0.02;
                self.activity_strings = self.activity_strings.at_least(0.0);
            }
        }

        match self.rx_conn_events.try_recv() {
            Ok(data) => {
                self.connection_events = data;
            }
            Err(_err) => {
            }
        }
    }

    pub fn get_conn_events(&mut self) {
        let tx = self.tx_conn_events.clone();
        let request = ehttp::Request::get("http://127.0.0.1:9000/connection_events");

        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            let Ok(response) = result else {
                warn!("Could not fetch msg_events");
                return;
            };

            let Ok(data) = serde_json::from_slice::<Vec<ConnEventData>>(&response.bytes) else {
                warn!("Could not decode data");
                return;
            };

            let _ = tx.send(data);
        })
    }

    pub fn get_msg_events(&mut self) {
        let tx = self.tx_msg_events.clone();
        let request = ehttp::Request::get("http://127.0.0.1:9000/msg_events");

        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            let Ok(result) = result else {
                warn!("Could not fetch msg_events");
                return;
            };

            let Ok(data) = serde_json::from_slice::<Vec<DataEvent>>(result.bytes.as_slice()) else {
                warn!("could not decode data");
                return;
            };

            // Convert the data to be something usable
            let mut map = HashMap::new();
            for event in data {
                match event {
                    DataEvent::PubMsg { id, utc_time } => {
                        let event_data = MsgEventData {
                            publish_time_ms: utc_time,
                            ack_time_ms: None,
                            id,
                        };
                        let _ = map.insert(id, event_data);
                    }
                    DataEvent::AckMsg {
                        id,
                        utc_time,
                        success,
                    } => {
                        if let Some(event_data) = map.get(&id) {
                            let mut event_data = event_data.clone();
                            if success {
                                event_data.ack_time_ms = Some(utc_time);
                            }
                            let _ = map.insert(id, event_data);
                        }
                    }
                }
            }
            let msg_event_data = map.into_values().collect();
            let _ = tx.send(msg_event_data);
        });
    }
}
