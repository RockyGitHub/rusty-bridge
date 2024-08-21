use rusty_bridge::{
    error::{Result, RustyBridgeError},
    initialize::initialize,
    main_loop::main_loop,
    shutdown::{shutdown, termination::ExitReason},
    title::{print_exit_title, print_title3},
};
use tracing::error;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    print_title3();
    println!("Version: [{}]", PKG_VERSION);

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(PKG_NAME));

    exit(result);
}

async fn run(package_name: &str) -> Result<()> {
    println!("Starting {}", package_name);

    // Initialize required objects
    let (adapter, data_source, transform, config_data, rx_new_msg, metrics_events, shutdown_token) =
        initialize()
            .await
            .map_err(|err| RustyBridgeError::Initialization(format!("{:?}", err)))?;

    // Run the main loop
    main_loop(metrics_events, adapter, rx_new_msg, shutdown_token).await;

    // Perform any shutdown logic
    shutdown().await;

    Ok(())
}

fn exit(result: core::result::Result<(), RustyBridgeError>) {
    println!("Exiting with result: [{:?}]", result);
    let exit_reason = match result {
        Ok(_) => ExitReason::Success,
        Err(err) => {
            error!("Exiting with error: [{}]", err);
            err.into()
        }
    };

    print_exit_title();
    // Using std::process::exit() lets us return more than just 0 or 1 values
    // This can be useful for the entrypoint.sh script
    std::process::exit(exit_reason.into());
}
