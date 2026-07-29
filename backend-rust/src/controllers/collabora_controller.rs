use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use axum::body::Bytes;
use crate::AppState;
use crate::dto::{CheckFileInfo, TokenQuery, JwtClaims};
use crate::helpers::{jwt, wopi};
use crate::repos::document_repo;

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

fn user_or_401(headers: &HeaderMap) -> Result<JwtClaims, Response> {
    jwt::extract_user(headers, JWT_SECRET)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

async fn wopi_verify(
    state: &AppState,
    query: &TokenQuery,
    doc_id: &str,
) -> Result<JwtClaims, Response> {
    let token = query.access_token.as_ref().ok_or_else(|| (StatusCode::UNAUTHORIZED, "No access token").into_response())?;
    let claims = jwt::validate(JWT_SECRET, token).ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid token").into_response())?;
    if claims.file_id != doc_id {
        return Err((StatusCode::FORBIDDEN, "Token/file mismatch").into_response());
    }
    Ok(claims)
}

pub async fn session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match crate::services::collabora_service::create_session(
        &state.db, &state.db_path, &headers, &id, &user.sub, &user.name, &state.collab_browser_prefix,
    ) {
        Ok(session) => Json(session).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

pub async fn check_file_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let doc = match document_repo::get_by_id(&state.db, &id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    Json(CheckFileInfo {
        base_file_name: format!("{}.{}", doc.name, doc.ext),
        size: doc.size,
        user_id: claims.sub,
        user_friendly_name: claims.name,
        version: "1.0".to_string(),
        last_modified_time: doc.updated_at,
        user_can_write: true,
        user_can_not_write_relative: false,
        breadcrumb_doc_name: doc.name,
        supports_locks: true,
        supports_get_lock: true,
        supports_update: true,
        supports_extended_lock_length: false,
    }).into_response()
}

pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match document_repo::read_file(&state.db_path, &id) {
        Some(content) => {
            let mime = document_repo::get_by_id(&state.db, &id)
                .map(|d| d.mime).unwrap_or_else(|| "application/octet-stream".to_string());
            ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

pub async fn file_ops(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let op = match wopi::wopi_override(&headers) {
        Some(op) => op,
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    };
    let lock_val = wopi::wopi_lock_value(&headers);

    match op.as_str() {
        "LOCK" | "REFRESH_LOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &id) {
                Some(existing) if existing != requested => wopi::conflict_response(Some(existing)),
                _ => { wopi::set_lock(&state.wopi_locks, &id, requested.clone()); wopi::lock_response(&requested) }
            }
        }
        "UNLOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &id) {
                Some(existing) if existing == requested => { wopi::clear_lock(&state.wopi_locks, &id); StatusCode::OK.into_response() }
                Some(existing) => wopi::conflict_response(Some(existing)),
                None => StatusCode::OK.into_response(),
            }
        }
        "GET_LOCK" => match wopi::current_lock(&state.wopi_locks, &id) {
            Some(lock) => wopi::lock_response(&lock),
            None => StatusCode::OK.into_response(),
        },
        _ => (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", op)).into_response(),
    }
}

pub async fn put_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
    body: Bytes,
) -> Response {
    let claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match wopi::wopi_override(&headers).as_deref() {
        Some("PUT") => {}
        Some(other) => return (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", other)).into_response(),
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    }

    if let Some(current) = wopi::current_lock(&state.wopi_locks, &id) {
        match wopi::wopi_lock_value(&headers) {
            Some(requested) if requested == current => {}
            Some(_) => return wopi::conflict_response(Some(current)),
            None => return wopi::conflict_response(Some(current)),
        }
    }

    let doc = document_repo::get_by_id(&state.db, &id).unwrap();
    document_repo::write_file(&state.db_path, &doc, &body).unwrap_or_default();
    document_repo::update_content(&state.db, &id, body.len() as u64);

    let mut response = StatusCode::OK.into_response();
    response.headers_mut().insert("X-WOPI-ItemVersion",
        HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0")));
    response
}
