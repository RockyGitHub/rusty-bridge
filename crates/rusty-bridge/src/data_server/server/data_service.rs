use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use http_body_util::Full;
use hyper::{body::Bytes, service::Service, Request, Response};
use tracing::trace;

use super::{
    data_events::{ConnectionEvent, DataEvent},
    request_handlers::{connection_events, health, msg_events},
};

//type Callback = fn(String) -> Result<Response<Full<Bytes>>, hyper::Error>;

// Wrapper to insert callbacks into hashmap
pub struct Callback {
    cb: fn(&DataService) -> Result<Response<Full<Bytes>>, hyper::Error>,
}

#[derive(Clone)]
pub struct DataService {
    // How to make the hashmap only need a &str?
    pub path_map: Arc<HashMap<&'static str, Callback>>,
    pub connections: Arc<Mutex<Vec<ConnectionEvent>>>,
    pub msgs: Arc<Mutex<Vec<DataEvent>>>,
}

impl DataService {
    pub fn new() -> Self {
        let mut path_map = HashMap::new();
        path_map.insert("/health", Callback { cb: health });
        path_map.insert(
            "/connection_events",
            Callback {
                cb: connection_events,
            },
        );
        path_map.insert("/msg_events", Callback { cb: msg_events });

        DataService {
            path_map: Arc::new(path_map),
            connections: Arc::new(Mutex::new(Vec::new())),
            msgs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn handle_event(&mut self, event: DataEvent) {
        match event {
            DataEvent::ConnectionEvent(data) => self
                .connections
                .lock()
                .expect("connection event")
                .push(data),
            DataEvent::NewMsg { .. } => self.msgs.lock().expect("msg event").push(event),
            DataEvent::PubMsg { .. } => self.msgs.lock().expect("msg event").push(event),
            DataEvent::AckMsg { .. } => self.msgs.lock().expect("msg event").push(event),
        }
    }
}

// Handles what to do with an incoming request.  Changes should be needed here ever except to maybe handle an error response better
impl<IncomingBody> Service<Request<IncomingBody>> for DataService {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        trace!("Processing request: [{}] [{}]", req.method(), req.uri());

        let res = match self.path_map.get(req.uri().path()) {
            Some(cb) => (cb.cb)(self),
            // Return 404
            None => return Box::pin(async { mk_response("oh no! not found".into()) }),
        };

        Box::pin(async { res })
    }
}
