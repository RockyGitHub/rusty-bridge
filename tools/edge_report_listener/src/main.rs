use std::{
    borrow::Cow, collections::HashMap, sync::{Arc, RwLock}, time::Duration
};

use axum::{
    error_handling::HandleErrorLayer, extract::{self, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};
use serde::{Deserialize, Serialize};
use tower::{ServiceBuilder, BoxError};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

type SharedState = Arc<RwLock<AppState>>;

#[derive(Debug, Default)]
struct AppState {
    edges: HashMap<String, EdgeData>,
}

#[derive(Debug, Default)]
struct EdgeData {
    static_data: EdgeStaticData,
    dynamic_data: EdgeDynamicData,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let shared_state = SharedState::default();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/edge_report/static", post(edge_report_static))
        .route("/edge_report/dynamic", post(edge_report_dynamic))
        //.route("/edge_report", post(create_user))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(Arc::clone(&shared_state));

    // run our app with hyper, listening globally on port 8999
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

impl EdgeData {
    fn new(static_data: EdgeStaticData) -> EdgeData {
        EdgeData {
            static_data,
            dynamic_data: EdgeDynamicData::default(),
        }
    }
    fn update_dynamic(&mut self, new_data: EdgeDynamicData) {
        self.dynamic_data = new_data;
    }

    // Maybe an ip address changed or something?
    fn update_static(&mut self, new_data: EdgeStaticData) {
        self.static_data = new_data
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct EdgeStaticData {
    machine_id: String,
    processor_serial: String,
    system_name: String,
    local_ip: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct EdgeDynamicData {
    machine_id: String,
    processor_use: f32,
    memory_total: u64,
    memory_available: u64,
    temperature: f32,
}

// basic handler that responds with a static string
async fn root(State(state): State<SharedState>) -> String {
    match state.read() {
        Ok(read) => format!("{:?}", read.edges),
        Err(err) => {
            error!("Lock was poisoned [{}]", err);
            "Internal server error. lock poisoned".to_string()
        },
    }
}

async fn edge_report_static(State(state): State<SharedState>, extract::Json(payload): extract::Json<EdgeStaticData>) {
    info!("Incoming from [{:?}]", payload);
    // TODO - needs to bbe decoded
    let id = payload.machine_id.clone();
    match state.write() {
        Ok(mut state) => {
            match state.edges.get_mut(&id) {
                Some(edge_data) => edge_data.update_static(payload),
                None => {
                    let _ = state.edges.insert(id, EdgeData::new(payload));
                },
            }
        },
        Err(err) => error!("Lock was poisoned [{}]", err),
    }
}
async fn edge_report_dynamic(State(state): State<SharedState>, extract::Json(payload): extract::Json<EdgeDynamicData>) {
    info!("Incoming from [{:?}]", payload);
    // TODO - needs to bbe decoded
    let id = payload.machine_id.clone();
    match state.write() {
        Ok(mut state) => {
            match state.edges.get_mut(&id) {
                Some(edge_data) => edge_data.update_dynamic(payload),
                None => {
                    warn!("Dynamic data received without original static data!");
                    let _ = state.edges.insert(id, EdgeData::default());
                },
            }
        },
        Err(err) => error!("Lock was poisoned [{}]", err),
    }
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {error}")),
    )
}

