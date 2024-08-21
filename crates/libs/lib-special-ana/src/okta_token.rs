use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::{de::IntoDeserializer, Deserialize};

use crate::error::ErrorSpecialAnA;

const AUDIENCE: &'static [&'static str] = &["", ""];

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct SpecialOktaToken {
    pub ver: u32,
    pub jti: String,
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub cid: String,
    pub uid: String,
    pub scp: Vec<String>,
    pub auth_time: u64,
    pub sub: String,
    #[serde(rename = "groupId", deserialize_with = "empty_string_as_none")]
    pub group_id: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub region: Option<String>,
    #[serde(rename = "nodeId", deserialize_with = "empty_string_as_none")]
    pub node_id: Option<String>,
}

impl SpecialOktaToken {
    /// Decodes a raw string token into its type. This is needed to fetch groupId and nodeId in Factory Provisioned devices
    pub fn decode(raw_token: &str) -> Result<Self, ErrorSpecialAnA> {
        let header = decode_header(raw_token)?;
        let mut validation = Validation::new(header.alg);
        validation.set_audience(AUDIENCE);
        validation.insecure_disable_signature_validation();
        let token = decode::<SpecialOktaToken>(
            raw_token,
            // No decoding key
            &DecodingKey::from_secret("".as_bytes()),
            &validation,
        )?
        .claims;

        Ok(token)
    }
}

/// If serde finds an empty string, it returns it as a None value instead of Some("")
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}
