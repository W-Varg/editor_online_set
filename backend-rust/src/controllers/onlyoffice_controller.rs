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
use crate::services::{onlyoffice_converter, onlyoffice_service};

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
    // Status 2: documento listo para guardar (autoguardado/cierre).
    // Status 6: forcesave (p. ej. al pulsar el botón "Guardar" con `forcesave: true`).
    // En ambos casos se descarga el estado actual y se persiste en el backend.
    if status == 2 || status == 6 {
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
                            tracing::info!("Callback saved document {} ({} bytes, status {})", id, bytes.len(), status);
                        }
                    }
                }
                Err(e) => tracing::error!("Callback: failed to download from URL for doc {}: {}", id, e),
            }
        }
    }
    Json(serde_json::json!({"error": 0}))
}

#[utoipa::path(
    post,
    path = "/api/documents/{id}/force-save",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    request_body(content = crate::dto::onlyoffice::ForceSaveRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Guardado forzado solicitado y persistido (o sin cambios que guardar)."),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Documento no encontrado"),
        (status = 502, description = "No se pudo contactar el command service de ONLYOFFICE")
    ),
    tag = "OnlyOffice",
    summary = "Forzar guardado del documento en ONLYOFFICE",
    description = "Pide a ONLYOFFICE que guarde el estado actual del documento (forcesave) y espera a que el backend \
        persista el archivo actualizado antes de responder. Lo usa el plugin de previsualización para garantizar \
        que el PDF refleje la última versión editada."
)]
pub async fn force_save(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<crate::dto::onlyoffice::ForceSaveRequest>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let doc = match document_repo::get_by_id(&state.db, &id) {
        Some(doc) => doc,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };
    if !document_repo::is_owner(&state.db, &id, &user.sub) && !document_repo::is_shared_with(&state.db, &id, &user.sub) {
        return (StatusCode::FORBIDDEN, "Sin acceso al documento").into_response();
    }
    if doc.editor != "onlyoffice" || doc.status == "final" || doc.ext == "pdf" {
        return (StatusCode::BAD_REQUEST, "El forcesave solo aplica a documentos en edición con ONLYOFFICE").into_response();
    }
    if request.key.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Falta la clave de sesión del documento").into_response();
    }

    let updated_before = doc.updated_at.clone();
    if let Err(error) = onlyoffice_converter::force_save(&request.key).await {
        tracing::warn!("Forcesave fallido para {}: {}", id, error);
        return (StatusCode::BAD_GATEWAY, error).into_response();
    }

    // Espera a que el callback status 6 del forcesave haya persistido el archivo
    // (timeout ~10s). Si expira se responde igualmente: el forcesave ya se solicitó
    // y el documento quedará al día con el siguiente autoguardado.
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Some(current) = document_repo::get_by_id(&state.db, &id) {
            if current.updated_at != updated_before {
                tracing::info!("Forcesave persistido para {} (updated_at {})", id, current.updated_at);
                return Json(serde_json::json!({ "saved": true })).into_response();
            }
        }
    }
    tracing::warn!("Forcesave de {} no confirmó el guardado antes del timeout", id);
    Json(serde_json::json!({ "saved": true })).into_response()
}
