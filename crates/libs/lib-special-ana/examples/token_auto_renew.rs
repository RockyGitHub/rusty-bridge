use std::time::Duration;

use special_ana::SpecialAnA;
use tokio::select;

#[tokio::main]
async fn main() {
    // Init the tracing collector so we can observe library logs if we want to
    // Set RUST_LOG env var to change the value, I recommend trace level
    tracing_subscriber::fmt::init();

    let endpoint = "my.endpoint.com";
    let username = "username".to_string();
    let password = "password".to_string();
    let ana_handler = SpecialAnA::new_mqtt(endpoint, username, password, true).unwrap();

    let mut renewal = ana_handler.get_token_renewal();

    loop {
        select! {
            pass = renewal.password_updated() => {
                println!("new password! {:?}", pass);
            },
            _ = tokio::time::sleep(Duration::from_secs(1)) => (),
        }
    }
    //let idk = renewal.future.recv().await;
}
