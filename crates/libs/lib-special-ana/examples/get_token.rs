use std::time::Duration;

use special_ana::SpecialAnA;

#[tokio::main]
async fn main() {
    // Init the tracing collector so we can observe library logs if we want to
    // Set RUST_LOG env var to change the value, I recommend trace level
    tracing_subscriber::fmt::init();

    let endpoint = "https://my.endpoint.com";
    let username = "username".to_string();
    let password = "password".to_string();
    let mut ana_handler = SpecialAnA::new_mqtt(endpoint, username, password, false).unwrap();

    let res = ana_handler.fetch_token(false).await;

    println!("{:?}", res);

    // Check for expiration
    println!("{}", ana_handler.is_expired().await);

    // Simulate waiting for the token to expire
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("{}", ana_handler.is_expired().await);

    // I'm thinking now that fetch_token should return a copy of the token if it's not expired yet?
    // to protect runaway token fetches?  I think there's an implication to doing this though
    println!("{:?}", ana_handler.fetch_token(false).await);
}
