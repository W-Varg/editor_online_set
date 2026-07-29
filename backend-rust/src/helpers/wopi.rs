use std::collections::HashMap;
use std::sync::Mutex;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

pub type WopiLocks = Mutex<HashMap<String, String>>;

pub fn wopi_override(headers: &HeaderMap) -> Option<String> {
    headers.get("x-wopi-override").and_then(|v| v.to_str().ok()).map(|v| v.to_ascii_uppercase())
}

pub fn wopi_lock_value(headers: &HeaderMap) -> Option<String> {
    headers.get("x-wopi-lock").and_then(|v| v.to_str().ok()).map(|v| v.to_string())
}

pub fn current_lock(locks: &WopiLocks, id: &str) -> Option<String> {
    locks.lock().ok().and_then(|l| l.get(id).cloned())
}

pub fn set_lock(locks: &WopiLocks, id: &str, lock: String) {
    if let Ok(mut l) = locks.lock() { l.insert(id.to_string(), lock); }
}

pub fn clear_lock(locks: &WopiLocks, id: &str) {
    if let Ok(mut l) = locks.lock() { l.remove(id); }
}

pub fn lock_response(lock: &str) -> Response {
    let mut r = StatusCode::OK.into_response();
    r.headers_mut().insert("X-WOPI-Lock", HeaderValue::from_str(lock).unwrap_or_else(|_| HeaderValue::from_static("")));
    r
}

pub fn conflict_response(lock: Option<String>) -> Response {
    let mut r = (StatusCode::CONFLICT, "Lock mismatch").into_response();
    if let Some(l) = lock {
        r.headers_mut().insert("X-WOPI-Lock", HeaderValue::from_str(&l).unwrap_or_else(|_| HeaderValue::from_static("")));
    }
    r
}
