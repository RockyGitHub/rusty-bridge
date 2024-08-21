use std::{collections::HashMap, time::Duration};

use cloud_adapter_core::ConnectionLost;
use rumqttc::{AsyncClient, ConnAck, ConnectReturnCode, EventLoop, PubAck, SubAck};
use special_ana::SpecialAnA;
use tokio::{
    select, spawn,
    sync::{broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

pub fn spawn_looper(
    mut event_loop: EventLoop,
    mut rx_run: watch::Receiver<bool>,
    tx_connect: broadcast::Sender<ConnectReturnCode>,
    mut rx_ack_oneshot: mpsc::Receiver<(u32, oneshot::Sender<u32>)>,
    tx_conn_lost: watch::Sender<ConnectionLost>,
    credentials: crate::Credentials,
    ana: SpecialAnA,
    client: AsyncClient,
    shutdown: CancellationToken,
) -> JoinHandle<EventLoop> {
    let event_handle = spawn(async move {
        let mut ack_map: HashMap<u16, (u32, oneshot::Sender<u32>)> = HashMap::new();
        //let mut password_expiration = ana.get_expiration_future().await;
        let mut password_renewal = ana.get_token_renewal();
        // Before starting the event_loop get a password..
        // TODO - can this go a level above, instead? -- this is slightly nicer here because it unblocks the init..
        select! {
            _ = shutdown.cancelled() => (),
            new_pass = password_renewal.password_updated() => {
                let new_password = new_pass.unwrap();
                let username = credentials.username.clone();
                event_loop
                    .mqtt_options
                    .set_credentials(username, new_password);
            }
        }

        'main: loop {
            select! {
                _ = shutdown.cancelled() => break,
                // TODO - There's surely a better way to handle connect/disconnect?
                _ = rx_run.changed() => {
                    loop {
                        // If rx_run is false, the 'main loop will be blocked until it's true.  Used for rusty-bridge "connect" and "disconnect"
                        // Because this library will auto reconnect as long as the event_loop is polled
                        if *rx_run.borrow_and_update() == false {
                            debug!("Blocking event_loop for disconnection");
                            match rx_run.changed().await {
                                Ok(_) => if *rx_run.borrow_and_update() == true {
                                    debug!("Unblocking event_loop for connection");
                                    break
                                },
                                Err(_err) => break 'main,
                            }
                        } else {
                            break
                        }
                    }
                },
                new_password = password_renewal.password_updated() => {
                    match new_password {
                        Ok(new_password) => {
                            let username = credentials.username.clone();
                            trace!("New password. username: {} password: {}", credentials.username, new_password);
                            event_loop.mqtt_options.set_credentials(username, new_password);
                            // Do not use client.disconnect() as that will cause a deadlock while the .await waits for the event_loop to move. Or spawn a task..
                            client.try_disconnect().unwrap();
                        },
                        Err(err) => {
                            error!("Unexpected error in password token: [{}]", err);
                        },
                    }
                }
                //_ = password_expiration.cancelled() => {
                    //info!("HiveMQ password has expired, fetching a new one, then will reconnect with the new password");
                    //// Password expired, we need to fetch a new one and set it in the mqtt_options for the event_loop
                    //let username = credentials.username.clone();
                    //// TODO - don't block here until password is fetched..
                    //let password = loop {
                        //match ana.fetch_token(false).await {
                            //Ok(password) => break password,
                            //Err(err) => warn!("Could not get password from AnA. [{}]", err),
                        //}
                    //};
                    //trace!("username: {} password: {}", credentials.username, password);
                    //event_loop.mqtt_options.set_credentials(username, password);
                    //password_expiration = ana.get_expiration_future().await;
                    //// Do not use client.disconnect() as that will cause a deadlock while the .await waits for the event_loop to move. Or spawn a task..
                    //client.try_disconnect().unwrap();
                //},
                event_result = event_loop.poll() => {
                    match event_result {
                        Ok(event) => match event {
                            rumqttc::Event::Incoming(inc) => {
                                debug!("Incoming: [{:?}]", inc);
                                match inc {
                                    // We should only ever receive a PubAck after a Publish, which should only occur if a oneshot channel is registered to the pkid
                                    rumqttc::Packet::PubAck(PubAck { pkid }) => match ack_map.remove(&pkid)
                                    {
                                        Some((msg_id, oneshot)) => {
                                            if let Err(err) = oneshot.send(msg_id) {
                                                debug!("Could not pass PubAck [{}] back to AckToken, token may have been dropped [{}]", pkid, err)
                                            }
                                        }
                                        None => warn!("pkid [{}] was not in the ack_token map", pkid),
                                    },
                                    rumqttc::Packet::ConnAck(ConnAck {
                                        session_present: _,
                                        code,
                                    }) => {
                                        let res = tx_connect.send(code);
                                        println!("res {:?}", res);
                                    }
                                    rumqttc::Packet::SubAck(SubAck { pkid, return_codes }) => {
                                        info!("SubAck: id: [{}], code: [{:?}]", pkid, return_codes)
                                    }
                                    rumqttc::Packet::Connect(_) => todo!(),
                                    rumqttc::Packet::Publish(_) => todo!(),
                                    rumqttc::Packet::PubRec(_) => todo!(),
                                    rumqttc::Packet::PubRel(_) => todo!(),
                                    rumqttc::Packet::PubComp(_) => todo!(),
                                    rumqttc::Packet::Subscribe(_) => todo!(),
                                    rumqttc::Packet::Unsubscribe(_) => todo!(),
                                    rumqttc::Packet::UnsubAck(_) => todo!(),
                                    rumqttc::Packet::PingReq => todo!(),
                                    rumqttc::Packet::PingResp => (),
                                    rumqttc::Packet::Disconnect => (),
                                    //_ => (),
                                }
                            }
                            rumqttc::Event::Outgoing(out) => {
                                debug!("Outgoing: [{:?}]", out);
                                match out {
                                    rumqttc::Outgoing::Publish(pkt_id) => match rx_ack_oneshot.try_recv() {
                                        Ok(oneshot) => match ack_map.insert(pkt_id, oneshot) {
                                            Some(oneshot) => drop(oneshot),
                                            None => (),
                                        },
                                        Err(err) => {
                                            error!("No oneshot available, publish mismatch! [{}]", err)
                                        }
                                    }, //todo! add this id to a hashmap to help with token delivery?
                                    rumqttc::Outgoing::Subscribe(_) => (),
                                    rumqttc::Outgoing::Unsubscribe(_) => todo!(),
                                    rumqttc::Outgoing::PubAck(_) => todo!(),
                                    rumqttc::Outgoing::PubRec(_) => todo!(),
                                    rumqttc::Outgoing::PubRel(_) => todo!(),
                                    rumqttc::Outgoing::PubComp(_) => todo!(),
                                    rumqttc::Outgoing::PingReq => (),
                                    rumqttc::Outgoing::PingResp => (),
                                    rumqttc::Outgoing::Disconnect => (),
                                    rumqttc::Outgoing::AwaitAck(_) => todo!(),
                                }
                            }
                        },
                        Err(err) => {
                            warn!("Connection lost: [{}]", err);
                            ack_map.clear();
                            match err {
                                rumqttc::ConnectionError::MqttState(_) => {
                                    tx_conn_lost.send_replace(ConnectionLost::ManualDisconnect)
                                }
                                rumqttc::ConnectionError::NetworkTimeout => {
                                    tx_conn_lost.send_replace(ConnectionLost::Timeout)
                                }
                                err => tx_conn_lost
                                    .send_replace(ConnectionLost::Uncategorized(err.to_string())),
                            };
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                        }
                    }
                }
            }
            //match event_loop.poll().await {}
        }

        event_loop
    });

    event_handle
}

//fn handle_event(&ack_map: HashMap)
