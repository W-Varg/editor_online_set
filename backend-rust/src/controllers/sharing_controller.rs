use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::AppState;
use crate::helpers::jwt;
use crate::repos::user_repo;
use crate::services::sharing_service;

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, JWT_SECRET)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SharePayload {
    pub user_id: String,
}

#[utoipa::path(
    get,
    path = "/api/users/search",
    params(
        ("q" = String, Query, description = "Text to search")
    ),
    responses(
        (status = 200, description = "Matching users", body = [crate::dto::UserSearchResult]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing"
)]
pub async fn search_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    Json(user_repo::search(&state.db, &query.q, &user.sub)).into_response()
}

#[utoipa::path(
    post,
    path = "/api/documents/{id}/shares",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    request_body = SharePayload,
    responses(
        (status = 201, description = "Share created", body = crate::dto::ShareResponse),
        (status = 400, description = "Invalid share request"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing"
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<SharePayload>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match sharing_service::share(&state.db, &id, &payload.user_id, &user.sub) {
        Ok(share) => (StatusCode::CREATED, Json(share)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/shares",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Current shares", body = [crate::dto::ShareResponse]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing"
)]
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let _user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    Json(sharing_service::list(&state.db, &id)).into_response()
}

#[utoipa::path(
    delete,
    path = "/api/documents/{id}/shares/{user_id}",
    params(
        ("id" = String, Path, description = "Document id"),
        ("user_id" = String, Path, description = "User id to unshare")
    ),
    responses(
        (status = 200, description = "Share removed"),
        (status = 404, description = "Share not found"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing"
)]
pub async fn remove(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, user_id)): Path<(String, String)>,
) -> Response {
    let _user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match sharing_service::remove(&state.db, &id, &user_id) {
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}
