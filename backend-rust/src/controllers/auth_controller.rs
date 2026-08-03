use std::sync::Arc;
use axum::{extract::State, Json};
use axum::http::{HeaderMap, StatusCode};
use crate::AppState;
use crate::dto::LoginRequest;
use crate::helpers::{config, jwt};
use crate::services::auth_service;
use crate::repos::user_repo;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Autenticación exitosa. Devuelve el token JWT y los datos del usuario.", body = crate::dto::AuthResponse),
        (status = 401, description = "Credenciales inválidas")
    ),
    tag = "Auth",
    summary = "Iniciar sesión",
    description = "Autentica al usuario con nombre de usuario y contraseña, y devuelve un token JWT de acceso. \
        Use el token como `Authorization: Bearer <token>` en el resto de endpoints."
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<crate::dto::AuthResponse>, (StatusCode, String)> {
    auth_service::login(&state.db, &req)
        .map(Json)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Credenciales inválidas".to_string()))
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "Listado de usuarios activos (solo lectura).", body = [crate::dto::UserSearchResult]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Auth",
    summary = "Listar usuarios",
    description = "Devuelve todos los usuarios activos del sistema. Requiere autenticación."
)]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::dto::UserSearchResult>>, (StatusCode, String)> {
    if jwt::extract_user(&headers, &config::jwt_secret()).is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Token requerido".to_string()));
    }
    Ok(Json(user_repo::list(&state.db)))
}
