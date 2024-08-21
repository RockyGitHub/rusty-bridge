mod hardcoded_special_transform;
use std::{borrow::Cow, collections::HashMap, sync::Arc, time::Duration};

use axum::{
    //debug_handler,
    async_trait,
    error_handling::HandleErrorLayer,
    extract::{self, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use data_source_core::{DataSourceInterface, TxData};
use serde::{Deserialize, Serialize};
use tokio::{spawn, task::JoinHandle};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, trace};

use crate::hardcoded_special_transform::SpecialEnvelope;

//type SharedState = Arc<RwLock<AppState>>;
type SharedState = Arc<AppState>;

#[derive(Serialize, Deserialize)]
pub struct Config {
    bind_address: String,
}

pub struct DataSourceHttpRest {
    server_handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct AppState {
    // Any data that I want to access within a route call
    tx_to_mini_edge: TxData,
}

#[async_trait]
impl DataSourceInterface for DataSourceHttpRest {
    async fn new_data_source(
        tx_new_data: TxData,
        config: &str,
    ) -> data_source_core::Result<DataSourceHttpRest> {
        let config = serde_json::from_str(config)
            .map_err(|err| data_source_core::error::Error::Initialize(err.to_string()))?;
        let Config { bind_address } = config;

        let shared_state = AppState {
            tx_to_mini_edge: tx_new_data,
        };
        let shared_state = Arc::new(shared_state);
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(root))
            .route("/health", get(health))
            .route("/data_in", post(data_in))
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(handle_error))
                    .load_shed()
                    .concurrency_limit(1024)
                    .timeout(Duration::from_secs(10))
                    .layer(TraceLayer::new_for_http()),
            )
            .with_state(Arc::clone(&shared_state));

        let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

        info!("Data-source listening on {}/data_in", bind_address);

        let handle = spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let data_source = DataSourceHttpRest {
            server_handle: handle,
        };

        Ok(data_source)
    }
}

async fn root(State(state): State<SharedState>) -> String {
    //match state.read() {
    //Ok(read) => format!("{:?}", read.edges),
    //Err(err) => {
    //error!("Lock was poisoned [{}]", err);
    //"Internal server error. lock poisoned".to_string()
    //}
    //}
    "todo".to_string()
}

async fn health(State(state): State<SharedState>) -> String {
    "Healthy!".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataIn {
    data: String,
    metadata: Option<HashMap<String, String>>,
}

//#[debug_handler]
async fn data_in(State(state): State<SharedState>, extract::Json(payload): extract::Json<DataIn>) {
    trace!("Data received at data-source");
    debug!("Data received at data-source [{:?}]", payload);

    // hardcoding this to an special type for brian tucker and I's tech challenge. Eventually this should not transform into an special envelope type
    let data = payload.data;
    let data = match serde_json::to_string(&SpecialEnvelope::new(data)) {
        Ok(data) => data,
        Err(err) => {
            error!("Could not convert envelope to string. [{}]", err);
            return;
        }
    };

    state.tx_to_mini_edge.send(data.as_bytes());
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
