#![crate_name = "special_ana"]
//! # Specal AnA
//!
//! `special_ana` is a library to handle fetching various tokens from AnA
//!
//! AnA - Authentication and Authorization
//!
//! ## How to use
//!
//! ```
//! let ana = SpecialAnA::new_mqtt("ana.endpoint.com", "000001", "amazingPassword", false);
//! let token = ana.fetch_token();
//!
//! if ana.is_expired() {
//!     println!("token is expired!")
//! }
//!
//! let future = ana.get_expiration_future();
//! tokio::spawn(async move {
//!     future.await();
//!     let new_token = ana.fetch_token();
//! })
//! ```
//!
mod error;
mod okta_token;
mod token_updated;

use std::{collections::HashMap, sync::Arc, time::Duration};

use error::ErrorSpecialAnA;
use serde::Deserialize;
use token_updated::TokenUpdatedPassword;
use tokio::{
    spawn,
    sync::{broadcast, RwLock},
    task::JoinHandle,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, trace, warn};
use url::Url;

//pub type RenewalToken = TokenUpdated<Result<String, ErrorSpecialAnA>>;

pub struct SpecialAnA {
    /// Current token
    token: Arc<RwLock<String>>,
    /// Can be cloned and awaited to signal the expiration of a token
    expiration_token: Arc<RwLock<CancellationToken>>,
    /// When a new token is fetched, this channel will send it to all [password_renewal](crate::SpecialAna::get_token_renewal) tokens
    updated_password: broadcast::Sender<String>,

    // Cached data
    endpoint: Url,
    token_post_body: HashMap<String, String>,

    // Cleanup
    expiration: Arc<RwLock<JoinHandle<()>>>,
    auto_renewal: Option<JoinHandle<()>>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct HiveMqAuthorizeResponse {
    token_type: String,
    expires_in: u64,
    access_token: String,
}

impl SpecialAnA {
    // TODO - don't pass auto_renew as a bool, make it a function that will start the renew task
    pub fn new_mqtt(
        ana_endpoint: &str,
        username: String,
        password: String,
        auto_renew: bool,
    ) -> Result<SpecialAnA, ErrorSpecialAnA> {
        let token = Arc::new(RwLock::new("".to_string()));
        let end_point = Url::parse(ana_endpoint)
            .map_err(|err| ErrorSpecialAnA::Initialization(err.to_string()))?;
        let expiration_task = tokio::time::sleep(Duration::from_secs(0));
        let handle = spawn(async move { expiration_task.await });
        let expiration_task = Arc::new(RwLock::new(handle));
        let mut token_post_body = HashMap::new();
        token_post_body.insert("username".to_string(), username);
        token_post_body.insert("password".to_string(), password);
        token_post_body.insert("scope".to_string(), "mqtt".to_string());
        let expiration_token = CancellationToken::new();
        expiration_token.cancel();
        let expiration_token = Arc::new(RwLock::new(expiration_token));

        let (tx, _) = broadcast::channel::<String>(1);

        let mut auto_renew_handle = None;
        if auto_renew {
            let endpoint = end_point.clone();
            let json_body = token_post_body.clone();
            let tx = tx.clone();
            let token = token.clone();
            let expiration_task = expiration_task.clone();
            let expiration_token = expiration_token.clone();
            let handle = spawn(async move {
                loop {
                    // Retry until success..
                    let response = loop {
                        match request_token(endpoint.clone(), &json_body).await {
                            Ok(response) => break response,
                            Err(err) => warn!("Could not fetch new token from AnA. [{}]", err),
                        }
                        // TODO - use a real retry algo with expontential backoff
                        sleep(Duration::from_secs(5)).await;
                    };

                    let HiveMqAuthorizeResponse {
                        token_type: _,
                        expires_in,
                        access_token,
                    } = response;

                    // wait for expiration time
                    info!(
                        "New token retrieved. Time till next token: [{}] seconds",
                        expires_in
                    );

                    set_internals_on_new_token(
                        &token,
                        &expiration_token,
                        &expiration_task,
                        access_token.clone(),
                        expires_in,
                    )
                    .await;

                    // Pass the token along to any RenewalTokens that may have been made
                    let _ = tx.send(access_token);

                    // In auto_renewal, attempt to get a new token before the current one expires
                    let expiration = response.expires_in - 30;
                    // TODO - use the cancel token here instead..
                    tokio::time::sleep(Duration::from_secs(expiration)).await;
                }
            });

            auto_renew_handle = Some(handle);
        }

        Ok(SpecialAnA {
            token: token,
            expiration: expiration_task,
            endpoint: end_point,
            token_post_body: token_post_body,
            expiration_token: expiration_token,
            updated_password: tx,
            auto_renewal: auto_renew_handle,
        })
    }

    /// Returns true or false if the last fetched token has expired
    ///
    /// # Examples
    ///
    /// ```
    /// let ana = SpecialAna::new_mqtt("ana.endpoint.com", "user", "pass");
    /// let _ = ana.fetch_token(false).await;
    /// assert_eq!(false, ana.is_expired());
    /// ```
    pub async fn is_expired(&self) -> bool {
        self.expiration_token.read().await.is_cancelled()
        //if self.expiration.is_finished() {
        //true
        //} else {
        //false
        //}
    }

    /// Gets the currently held token, expired or not
    pub async fn get_token(&self) -> String {
        self.token.read().await.clone()
    }

    pub async fn get_expiration_future(&self) -> CancellationToken {
        self.expiration_token.read().await.child_token()
        //self.expiration_token.read().await.clone()
    }

    /// Returns a RenewalToken which can be awaited to receive a new password if `auto_renew` is true
    pub fn get_token_renewal(&self) -> TokenUpdatedPassword {
        let rx = self.updated_password.subscribe();
        //let idk = async move { Ok(String) };
        TokenUpdatedPassword { rx }
    }

    /// Gets the currently active token or fetches a new one if it is expired
    ///
    /// # Parameters
    ///
    /// * `force` - bypasses the expiration check and fetches a new token. Tokens cost money, so be careful this doesn't occur often
    ///
    /// # Returns
    ///
    /// * `&str`` of okta token
    pub async fn fetch_token(&mut self, force: bool) -> Result<String, ErrorSpecialAnA> {
        // If not expired and not forcing a new token, return the existing one
        if !self.is_expired().await && !force {
            return Ok(self.get_token().await);
        }

        let response = request_token(self.endpoint.clone(), &self.token_post_body).await?;

        // We have to make the assumption this is passed in seconds, the variable name doesn't specify the unit
        let expiration_time = response.expires_in;
        let new_token = response.access_token;

        set_internals_on_new_token(
            &self.token,
            &self.expiration_token,
            &self.expiration,
            new_token.clone(),
            expiration_time,
        )
        .await;

        // Handle the token_renewal tokens by passing the new token to them
        let _ = self.updated_password.send(new_token.clone());

        info!(
            "New Okta token retreived, expiration in [{}] seconds",
            expiration_time
        );
        Ok(new_token)
    }
}

// Updating AnA can happen from two places, either in `fetch_token` or in the auto_renewal task. This consolidates what changes to the same call
async fn set_internals_on_new_token(
    token: &Arc<RwLock<String>>,
    expiration_token: &Arc<RwLock<CancellationToken>>,
    expiration_task: &Arc<RwLock<JoinHandle<()>>>,
    new_token: String,
    expiration_time_s: u64,
) {
    *token.write().await = new_token;

    // Handle the expiration token(s) and set a new one
    let new_expiration_token = CancellationToken::new();
    *expiration_token.write().await = new_expiration_token.child_token();
    *expiration_task.write().await = spawn(async move {
        tokio::time::sleep(Duration::from_secs(expiration_time_s)).await;
        //tokio::time::sleep(Duration::from_secs(10)).await; // TEST
        new_expiration_token.cancel()
    });
}

async fn request_token<T>(
    endpoint: Url,
    json_body: &T,
) -> Result<HiveMqAuthorizeResponse, ErrorSpecialAnA>
where
    T: serde::Serialize + ?Sized,
{
    let client = reqwest::Client::new();

    let res = client
        .post(endpoint)
        .json(json_body)
        .send()
        .await
        .map_err(|err| ErrorSpecialAnA::FetchingToken(err.to_string()))?;

    let text = res
        .text()
        .await
        .map_err(|err| ErrorSpecialAnA::FetchingToken(err.to_string()))?;
    trace!("AnA response text: [{}]", text);

    let response = serde_json::from_str::<HiveMqAuthorizeResponse>(&text).map_err(|err| {
        ErrorSpecialAnA::FetchingToken(format!(
            "could not deserialize: [{}], text: [{}]",
            err, text
        ))
    })?;

    Ok(response)
}

#[allow(dead_code)]
fn test_example() {}

impl Drop for SpecialAnA {
    fn drop(&mut self) {
        trace!("Dropping SpecialAnA, awaiting handle drop");
        //self.expiration.try_write().abort();
        if let Some(renewal_handle) = self.auto_renewal.take() {
            renewal_handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = test_example();
        assert_eq!(result, ());
    }
}
