use std::sync::Arc;
use axum::{extract::State, Json};
use axum::http::StatusCode;
use crate::AppState;
use crate::dto::LoginRequest;
use crate::services::auth_service;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authenticated", body = crate::dto::AuthResponse),
        (status = 401, description = "Credenciales inválidas")
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<crate::dto::AuthResponse>, (StatusCode, String)> {
    auth_service::login(&state.db, &req)
        .map(Json)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Credenciales inválidas".to_string()))
}
