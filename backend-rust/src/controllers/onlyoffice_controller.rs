use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::AppState;
use crate::helpers::{config, jwt};
use crate::repos::document_repo;
use crate::services::onlyoffice_service;

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[utoipa::path(
    get,
    path = "/api/onlyoffice/config/{id}",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Configuración para inicializar el editor de ONLYOFFICE.", body = crate::dto::OnlyOfficeConfig),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "OnlyOffice",
    summary = "Obtener configuración de ONLYOFFICE",
    description = "Genera la configuración (documento, editorConfig, plugins y token JWT) para abrir un documento \
        en el editor de ONLYOFFICE. Requiere autenticación."
)]
pub async fn config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let api_token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or("");
    match onlyoffice_service::get_config(&state.db, &state.db_path, &headers, &id, &user.sub, &user.name, api_token) {
        Some(mut config) => {
            let config_json = serde_json::to_value(&config).unwrap();
            let jwt_token = encode(
                &Header::default(),
                &config_json,
                &EncodingKey::from_secret(config::jwt_secret().as_bytes()),
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    request_body(content = serde_json::Value, content_type = "application/json"),
    responses(
        (status = 200, description = "Callback aceptado. Devuelve `{\"error\": 0}`.")
    ),
    tag = "OnlyOffice",
    summary = "Callback de guardado de ONLYOFFICE",
    description = "Recibe la notificación de ONLYOFFICE al guardar/cerrar el documento y descarga el contenido actualizado."
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
