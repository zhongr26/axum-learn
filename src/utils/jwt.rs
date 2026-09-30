use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: String, // username
  pub exp: u64,
  pub fresh: bool,
}

pub fn sign(username: &str, secret: &str, ttl_secs: i64) -> anyhow::Result<String> {
  let claims = Claims {
    sub: username.into(),
    exp: (chrono::Utc::now() + chrono::Duration::seconds(ttl_secs)).timestamp() as u64,
    fresh: true,
  };

  Ok(encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )?)
}

pub fn verify(token: &str, secret: &str) -> Option<Claims> {
  decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )
  .ok()
  .map(|d| d.claims)
}
