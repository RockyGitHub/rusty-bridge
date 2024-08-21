use core::time;
use std::thread;

use tokio::{runtime::Handle, select, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::error::RustyBridgeError;

const SHUTDOWN_TIMEOUT_MS: u64 = 10_000;

pub fn register_shutdown_signals(
    cancel_token: CancellationToken,
) -> Result<JoinHandle<()>, RustyBridgeError> {
    #[cfg(target_os = "linux")]
    let handle = linux(cancel_token)?;
    #[cfg(target_os = "windows")]
    let handle = windows(cancel_token)?;

    Ok(handle)
}

#[cfg(target_os = "linux")]
fn linux(cancel_token: CancellationToken) -> Result<JoinHandle<()>, RustyBridgeError> {
    use tokio::signal::unix::{signal, SignalKind};

    // Register to various interrupt types
    let mut interrupt = signal(SignalKind::interrupt()).map_err(|err| {
        RustyBridgeError::Initialization(format!("failed to register to SIGINT. [{}]", err))
    })?;
    let mut hangup = signal(SignalKind::hangup()).map_err(|err| {
        RustyBridgeError::Initialization(format!("failed to register to SIGHUP. [{}]", err))
    })?;
    let mut terminate = signal(SignalKind::terminate()).map_err(|err| {
        RustyBridgeError::Initialization(format!("failed to register to SIGTERM. [{}]", err))
    })?;

    // Listen and wait for activity amongst the different signals
    let handle = Handle::current().spawn(async move {
        select! {
            _ = interrupt.recv() => {
                warn!("SIGINT received");
                cancel_token.cancel()
            },
            _ = hangup.recv() => {
                warn!("SIGHUP received");
                cancel_token.cancel()
            },
            _ = terminate.recv() => {
                warn!("SIGTERM received");
                cancel_token.cancel()
            },
            //_ = cancel_token.cancelled() => (),
        }

        // Launch a backup thread incase the program hangs on shutdown. This will force a non-clean termination
        thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(SHUTDOWN_TIMEOUT_MS));
            error!(
                "Exceeded clean shutdown timeout [{}] ms, forcefully terminating",
                SHUTDOWN_TIMEOUT_MS
            );
            eprintln!(
                "Exceeded clean shutdown timeout [{}] ms, forcefully terminating",
                SHUTDOWN_TIMEOUT_MS
            );
            std::process::exit(-1)
        });
    });

    Ok(handle)
}

#[cfg(target_os = "windows")]
fn windows(cancel_token: CancellationToken) -> Result<JoinHandle<()>, RustyBridgeError> {
    use tokio::signal::ctrl_c;

    let signal = ctrl_c();

    // Listen and wait for activity amongst the different signals
    let handle = Handle::current().spawn(async move {
        select! {
            _ = signal => {
                warn!("Ctrl-C received");
                cancel_token.cancel()
            },
            //_ = cancel_token.cancelled() => (),
        }

        // Launch a backup thread incase the program hangs on shutdown. This will force a non-clean termination
        thread::spawn(move || {
            thread::sleep(time::Duration::from_millis(SHUTDOWN_TIMEOUT_MS));
            error!(
                "Exceeded clean shutdown timeout [{}] ms, forcefully terminating",
                SHUTDOWN_TIMEOUT_MS
            );
            eprintln!(
                "Exceeded clean shutdown timeout [{}] ms, forcefully terminating",
                SHUTDOWN_TIMEOUT_MS
            );
            std::process::exit(-1)
        });
    });

    Ok(handle)
}
