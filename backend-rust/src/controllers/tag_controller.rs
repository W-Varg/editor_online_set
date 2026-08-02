use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use crate::AppState;
use crate::helpers::{config, jwt};
use crate::services::tag_service;

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[utoipa::path(
    get,
    path = "/api/tags",
    responses(
        (status = 200, description = "Available document tags", body = [crate::dto::TagDefinition]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Tags"
)]
pub async fn list_tags(State(_state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(e) = user_or_401(&headers) {
        return e;
    }
    Json(tag_service::list()).into_response()
}

#[utoipa::path(
    get,
    path = "/api/preview-source/{token}",
    params(("token" = String, Path, description = "One-time token for resolved document content")),
    responses(
        (status = 200, description = "Resolved document content"),
        (status = 404, description = "Token no válido o expirado")
    ),
    tag = "Tags"
)]
pub async fn preview_source(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let mut store = state.preview_sources.lock().unwrap();
    match store.remove(&token) {
        Some(entry) => {
            if entry.expires_at < std::time::Instant::now() {
                return (StatusCode::NOT_FOUND, "Token expirado").into_response();
            }
            (
                [
                    (axum::http::header::CONTENT_TYPE, entry.mime.as_str()),
                    (axum::http::header::CACHE_CONTROL, "no-store"),
                ],
                entry.content,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Token no válido").into_response(),
    }
}
