use http_body_util::Full;
use hyper::{body::Bytes, Response};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{data_events::DataEvent, data_service::DataService};

// Functions to handle what to do with certain request paths
type HyperServiceReturn = Result<Response<Full<Bytes>>, hyper::Error>;
pub fn health(_: &DataService) -> HyperServiceReturn {
    Ok(Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from("Alive!")))
        .unwrap())
}

pub fn connection_events(data_service: &DataService) -> HyperServiceReturn {
    let data = data_service.connections.lock().expect("poisoned lock");

    let serialized = match serde_json::to_vec(&*data) {
        Ok(serialized) => serialized,
        Err(err) => {
            warn!(
                "Could not serialize data_service connection events data. [{}]",
                err
            );
            return Ok(Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .body(Full::new(Bytes::from(err.to_string())))
                .unwrap());
        }
    };

    Ok(Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(serialized)))
        .unwrap())
}

pub fn msg_events(data_service: &DataService) -> HyperServiceReturn {
    let msg_history = format!("{:?}", data_service.msgs.lock().expect("poisoned lock"));
    let data = data_service.msgs.lock().expect("poisoned lock");

    let serialized = match serde_json::to_vec(&*data) {
        Ok(serialized) => serialized,
        Err(err) => {
            warn!(
                "Could not serialize data_service msg events data. [{}]",
                err
            );
            // TODO - this isn't right
            return Ok(Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .body(Full::new(Bytes::from("data serialization failed")))
                .unwrap());
        }
    };

    Ok(Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(serialized)))
        .unwrap())
}
