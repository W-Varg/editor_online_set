use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::AppState;
use crate::dto::{CreateTemplate, RenameTemplate};
use crate::helpers::{config, jwt};
use crate::models::Document;
use crate::repos::template_repo;
use crate::services::{converter, onlyoffice_converter, onlyoffice_service, template_service};

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

fn template_as_document(t: &crate::models::Template) -> Document {
    Document {
        id: t.id.clone(),
        name: t.name.clone(),
        ext: t.ext.clone(),
        mime: t.mime.clone(),
        editor: t.editor.clone(),
        size: t.size,
        status: "draft".to_string(),
        owner_id: t.owner_id.clone(),
        created_at: t.created_at.clone(),
        updated_at: t.updated_at.clone(),
    }
}

fn pdf_response(content: Vec<u8>) -> Response {
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/pdf"),
            (axum::http::header::CONTENT_DISPOSITION, "inline"),
            (axum::http::header::CACHE_CONTROL, "no-store, max-age=0"),
        ],
        content,
    ).into_response()
}

#[utoipa::path(
    get,
    path = "/api/templates",
    responses(
        (status = 200, description = "Plantillas disponibles (globales).", body = [crate::dto::TemplateResponse]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Templates",
    summary = "Listar plantillas",
    description = "Devuelve las plantillas de documento disponibles para todos los usuarios. Requiere autenticación."
)]
pub async fn list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(e) = user_or_401(&headers) {
        return e;
    }
    let templates: Vec<serde_json::Value> = template_service::list(&state.db)
        .into_iter()
        .map(|t| serde_json::to_value(t).unwrap())
        .collect();
    Json(templates).into_response()
}

#[utoipa::path(
    post,
    path = "/api/templates",
    request_body = CreateTemplate,
    responses(
        (status = 201, description = "Plantilla creada.", body = crate::dto::TemplateResponse),
        (status = 400, description = "Solicitud inválida"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Templates",
    summary = "Crear plantilla",
    description = "Crea una plantilla en blanco o a partir del contenido de un documento existente (`source_document_id`). Requiere autenticación."
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTemplate>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match template_service::create(&state.db, &state.db_path, &payload, &user.sub) {
        Ok(template) => {
            let resp = serde_json::json!({
                "id": template.id, "name": template.name, "ext": template.ext,
                "mime": template.mime, "editor": template.editor, "size": template.size,
                "owner_id": template.owner_id, "owner_name": user.name,
                "created_at": template.created_at, "updated_at": template.updated_at,
            });
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/templates/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "Metadatos de la plantilla.", body = crate::dto::TemplateResponse),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Templates",
    summary = "Obtener plantilla",
    description = "Devuelve los metadatos de una plantilla por su id."
)]
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match template_service::get_by_id(&state.db, &id) {
        Some(t) => Json(t).into_response(),
        None => (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/templates/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    request_body = RenameTemplate,
    responses(
        (status = 200, description = "Plantilla renombrada.", body = crate::dto::TemplateResponse),
        (status = 400, description = "Solicitud inválida"),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Templates",
    summary = "Renombrar plantilla",
    description = "Cambia el nombre de una plantilla. Requiere autenticación."
)]
pub async fn rename(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RenameTemplate>,
) -> Response {
    if let Err(e) = user_or_401(&headers) {
        return e;
    }
    match template_service::rename(&state.db, &id, &payload.name) {
        Ok(t) => Json(t).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/templates/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "Plantilla eliminada."),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Templates",
    summary = "Eliminar plantilla",
    description = "Elimina la plantilla y su archivo en disco. Requiere autenticación."
)]
pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(e) = user_or_401(&headers) {
        return e;
    }
    match template_service::delete(&state.db, &id, &state.db_path) {
        Ok(()) => Json(serde_json::json!({"deleted": true})).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/templates/{id}/content",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "Contenido binario de la plantilla."),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Templates",
    summary = "Obtener contenido de plantilla",
    description = "Devuelve el contenido binario de la plantilla. Lo usan los editores y el convertidor."
)]
pub async fn content(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let template = match template_service::get_by_id(&state.db, &id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    };
    let content = match template_repo::read_file(&state.db_path, &id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    };
    ([(axum::http::header::CONTENT_TYPE, template.mime.as_str())], content).into_response()
}

#[utoipa::path(
    get,
    path = "/api/templates/{id}/preview",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "PDF temporal de la plantilla."),
        (status = 400, description = "La plantilla no es convertible"),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Plantilla no encontrada"),
        (status = 502, description = "Error del servicio de conversión")
    ),
    tag = "Templates",
    summary = "Previsualizar plantilla",
    description = "Genera un PDF temporal de la plantilla para la previsualización en el frontend. \
        Requiere autenticación."
)]
pub async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let template = match template_service::get_by_id(&state.db, &id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    };
    let content = match template_repo::read_file(&state.db_path, &id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    };
    let doc = template_as_document(&template);
    if !onlyoffice_converter::is_supported(&doc) {
        return (StatusCode::BAD_REQUEST, "La previsualización solo está disponible para plantillas Word o Excel").into_response();
    }
    let fallback_source_url = format!(
        "{}/api/templates/{}/content",
        config::public_backend_url(8091),
        id
    );
    match converter::content_to_pdf(&state, &doc, &content, &user.sub, &user.name, Some(fallback_source_url)).await {
        Ok(pdf) => pdf_response(pdf),
        Err(error) => {
            tracing::error!("Previsualización de plantilla fallida para {}: {}", id, error);
            (StatusCode::BAD_GATEWAY, error).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/onlyoffice/config/template/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "Configuración de ONLYOFFICE para abrir la plantilla.", body = crate::dto::OnlyOfficeConfig),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Templates",
    summary = "Obtener configuración de ONLYOFFICE (plantilla)",
    description = "Genera la configuración para abrir una plantilla en el editor de ONLYOFFICE. Requiere autenticación."
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
    match onlyoffice_service::get_template_config(
        &state.db,
        &state.db_path,
        &headers,
        &id,
        &user.sub,
        &user.name,
        api_token,
    ) {
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
        None => (StatusCode::NOT_FOUND, "Plantilla no encontrada").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/callback/template/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    request_body(content = serde_json::Value, content_type = "application/json"),
    responses(
        (status = 200, description = "Callback aceptado. Devuelve `{\"error\": 0}`.")
    ),
    tag = "Templates",
    summary = "Callback de guardado de plantilla",
    description = "Recibe la notificación de ONLYOFFICE al guardar/cerrar la plantilla y descarga el contenido actualizado."
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
                        if let Some(template) = template_repo::get_by_id(&state.db, &id) {
                            template_repo::write_file(&state.db_path, &template, &bytes).unwrap_or_default();
                            template_repo::update_size(&state.db, &id, bytes.len() as u64);
                            tracing::info!("Callback saved template {} ({} bytes)", id, bytes.len());
                        }
                    }
                }
                Err(e) => tracing::error!("Callback: failed to download from URL for template {}: {}", id, e),
            }
        }
    }
    Json(serde_json::json!({"error": 0}))
}
