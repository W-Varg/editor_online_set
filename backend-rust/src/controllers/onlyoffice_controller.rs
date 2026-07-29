use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::AppState;
use crate::helpers::jwt;
use crate::repos::document_repo;
use crate::services::onlyoffice_service;

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, JWT_SECRET)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[utoipa::path(
    get,
    path = "/api/onlyoffice/config/{id}",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "OnlyOffice configuration", body = crate::dto::OnlyOfficeConfig),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Document not found")
    ),
    tag = "OnlyOffice"
)]
pub async fn config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match onlyoffice_service::get_config(&state.db, &state.db_path, &headers, &id, &user.sub, &user.name) {
        Some(mut config) => {
            let config_json = serde_json::to_value(&config).unwrap();
            let jwt_token = encode(
                &Header::default(),
                &config_json,
                &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
            ).unwrap_or_default();
            config.token = Some(jwt_token);
            Json(config).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Document not found").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/callback/onlyoffice/{id}",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Callback accepted")
    ),
    tag = "OnlyOffice"
)]
pub async fn callback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let status = payload.get("status").and_then(|s| s.as_i64()).unwrap_or(0);
    let url = payload.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
    if status == 2 || status == 3 {
        if let Some(download_url) = url {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build().unwrap_or_default();
            match client.get(&download_url).send().await {
                Ok(resp) => {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Some(doc) = document_repo::get_by_id(&state.db, &id) {
                            document_repo::write_file(&state.db_path, &doc, &bytes).unwrap_or_default();
                            document_repo::update_content(&state.db, &id, bytes.len() as u64);
                            tracing::info!("Callback saved document {} ({} bytes)", id, bytes.len());
                        }
                    }
                }
                Err(e) => tracing::error!("Callback: failed to download from URL for doc {}: {}", id, e),
            }
        }
    }
    Json(serde_json::json!({"error": 0}))
}
