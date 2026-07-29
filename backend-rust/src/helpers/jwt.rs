use axum::http::HeaderMap;
use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use crate::dto::JwtClaims;

pub fn create(secret: &str, user_id: &str, user_name: &str, file_id: &str, ttl_seconds: i64) -> String {
    let claims = JwtClaims {
        sub: user_id.to_string(),
        file_id: file_id.to_string(),
        name: user_name.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds)).timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
}

pub fn validate(secret: &str, token: &str) -> Option<JwtClaims> {
    decode::<JwtClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256))
        .ok().map(|d| d.claims)
}

pub fn extract_user(headers: &HeaderMap, secret: &str) -> Option<JwtClaims> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    validate(secret, token)
}
