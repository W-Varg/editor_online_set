use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use crate::AppState;
use crate::dto::{CreateDocument, ConvertResponse};
use crate::helpers::{config, jwt, url};
use crate::services::{collabora_converter, document_service, onlyoffice_converter, tag_service};
use crate::repos::{document_repo, user_repo};

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[derive(Deserialize)]
pub struct DocQuery {
    pub tab: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/documents",
    params(
        ("tab" = Option<String>, Query, description = "mine or shared")
    ),
    responses(
        (status = 200, description = "Document list", body = [crate::dto::DocumentResponse]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Documents"
)]
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<DocQuery>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let tab = query.tab.as_deref().unwrap_or("mine");
    let docs: Vec<serde_json::Value> = match tab {
        "shared" => document_service::get_shared(&state.db, &user.sub)
            .into_iter().map(|d| serde_json::to_value(d).unwrap()).collect(),
        _ => document_service::get_mine(&state.db, &user.sub)
            .into_iter().map(|d| serde_json::to_value(d).unwrap()).collect(),
    };
    Json(docs).into_response()
}

#[utoipa::path(
    post,
    path = "/api/documents",
    request_body = CreateDocument,
    responses(
        (status = 201, description = "Document created", body = crate::dto::DocumentResponse),
        (status = 400, description = "Invalid document request"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Documents"
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateDocument>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match document_service::create(&state.db, &state.db_path, &payload, &user.sub) {
        Ok(doc) => {
            let resp = serde_json::json!({
                "id": doc.id, "name": doc.name, "ext": doc.ext,
                "mime": doc.mime, "editor": doc.editor, "size": doc.size,
                "status": doc.status, "owner_id": doc.owner_id,
                "owner_name": user.name, "created_at": doc.created_at,
                "updated_at": doc.updated_at,
            });
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Document", body = crate::dto::DocumentResponse),
        (status = 404, description = "Document not found")
    ),
    tag = "Documents"
)]
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match document_service::get_by_id(&state.db, &id) {
        Some(doc) => Json(doc).into_response(),
        None => (StatusCode::NOT_FOUND, "Document not found").into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/documents/{id}",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Delete result", body = crate::dto::DeleteResponse),
        (status = 401, description = "Token requerido"),
        (status = 500, description = "Delete failed")
    ),
    tag = "Documents"
)]
pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match document_service::delete(&state.db, &id, &user.sub, &state.db_path) {
        Ok(deleted) => Json(serde_json::json!({"deleted": deleted})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/download/{id}",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Raw document content"),
        (status = 404, description = "Not found")
    ),
    tag = "Documents"
)]
pub async fn download(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let doc = match document_service::get_by_id(&state.db, &id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    let content = match document_repo::read_file(&state.db_path, &id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    ([(axum::http::header::CONTENT_TYPE, doc.mime.as_str())], content).into_response()
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/content",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "Raw document content"),
        (status = 404, description = "Document not found")
    ),
    tag = "Documents"
)]
pub async fn content(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let content = match document_repo::read_file(&state.db_path, &id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };
    let mime = document_service::get_by_id(&state.db, &id)
        .map(|d| d.mime)
        .unwrap_or_else(|| "application/octet-stream".to_string());
    ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
}

#[utoipa::path(
    post,
    path = "/api/documents/{id}/convert",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "PDF conversion result", body = ConvertResponse),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Document not found")
    ),
    tag = "Documents"
)]
pub async fn convert_to_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let doc = match document_service::get_by_id(&state.db, &id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };

    if doc.owner_id != user.sub {
        return (StatusCode::FORBIDDEN, "Solo el propietario puede convertir el documento").into_response();
    }
    if doc.status == "final" {
        let pdf_url = format!("{}/api/documents/{}/pdf", url::public_service_url(&headers, 8091), id);
        return Json(ConvertResponse { pdf_id: format!("{}.pdf", id), pdf_url, status: "already_converted".to_string() }).into_response();
    }
    if !is_convertible(&doc) {
        return (StatusCode::BAD_REQUEST, "La conversión solo está disponible para documentos Word o Excel compatibles").into_response();
    }

    let content = match document_repo::read_file(&state.db_path, &id) {
        Some(content) => content,
        None => return (StatusCode::NOT_FOUND, "Contenido del documento no encontrado").into_response(),
    };

    let pdf_bytes = match convert_content_to_pdf(&state, &doc, &content, &user.sub, &user.name).await {
        Ok(pdf) => pdf,
        Err(error) => {
            tracing::error!("Conversión {} fallida para {}: {}", doc.editor, id, error);
            return (StatusCode::BAD_GATEWAY, error).into_response();
        }
    };

    match document_repo::finalize_pdf(&state.db, &id, &state.db_path, &pdf_bytes) {
        Ok(()) => {
            let pdf_url = format!("{}/api/documents/{}/pdf", url::public_service_url(&headers, 8091), id);
            Json(ConvertResponse { pdf_id: format!("{}.pdf", id), pdf_url, status: "converted".to_string() }).into_response()
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/preview",
    params(("id" = String, Path, description = "Document id")),
    responses((status = 200, description = "Temporary PDF preview"), (status = 401, description = "Token requerido")),
    tag = "Documents"
)]
pub async fn preview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let doc = match document_service::get_by_id(&state.db, &id) {
        Some(doc) => doc,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };
    if !document_repo::is_owner(&state.db, &id, &user.sub) && !document_repo::is_shared_with(&state.db, &id, &user.sub) {
        return (StatusCode::FORBIDDEN, "Sin acceso al documento").into_response();
    }

    if doc.status == "final" || doc.ext == "pdf" {
        return get_pdf(State(state), headers, Path(id)).await;
    }
    if !is_convertible(&doc) {
        return (StatusCode::BAD_REQUEST, "La previsualización solo está disponible para documentos Word o Excel compatibles").into_response();
    }
    let content = match document_repo::read_file(&state.db_path, &id) {
        Some(content) => content,
        None => return (StatusCode::NOT_FOUND, "Contenido del documento no encontrado").into_response(),
    };
    match convert_content_to_pdf(&state, &doc, &content, &user.sub, &user.name).await {
        Ok(pdf) => pdf_response(pdf, true),
        Err(error) => {
            tracing::error!("Previsualización {} fallida para {}: {}", doc.editor, id, error);
            (StatusCode::BAD_GATEWAY, error).into_response()
        }
    }
}

fn is_convertible(doc: &crate::models::Document) -> bool {
    match doc.editor.as_str() {
        "onlyoffice" => onlyoffice_converter::is_supported(doc),
        "collabora" => collabora_converter::is_supported(doc),
        _ => false,
    }
}

async fn convert_content_to_pdf(
    state: &Arc<AppState>,
    doc: &crate::models::Document,
    content: &[u8],
    user_id: &str,
    session_name: &str,
) -> Result<Vec<u8>, String> {
    // Resuelve las etiquetas {{key}} con los datos del usuario que previsualiza.
    // Si el usuario ya no existe en la DB (IDs regenerados por un reseed), se usa
    // el nombre de la sesión del JWT como fallback para no perder la resolución.
    let user = match user_repo::get_by_id(&state.db, user_id) {
        Some(user) => Some(user),
        None => Some(crate::models::User {
            id: user_id.to_string(),
            username: user_id.to_string(),
            name: session_name.to_string(),
            dni: None,
            cargo: None,
            email: None,
        }),
    };
    let resolved = user.and_then(|u| tag_service::resolve(content, &u, &doc.ext));
    let (conversion_content, source_url) = match (&resolved, doc.editor.as_str()) {
        // ONLYOFFICE fetchea el documento por URL (no recibe bytes): el contenido
        // resuelto se registra con un token de un solo uso en /api/preview-source.
        (Some(resolved), "onlyoffice") => {
            let token = uuid::Uuid::new_v4().to_string();
            let mime = doc.mime.clone();
            let url = {
                let mut store = state.preview_sources.lock().unwrap();
                store.insert(
                    token.clone(),
                    crate::PreviewSource {
                        content: resolved.clone(),
                        mime,
                        expires_at: std::time::Instant::now() + std::time::Duration::from_secs(60),
                    },
                );
                format!("{}/api/preview-source/{}", config::public_backend_url(8091), token)
            };
            (resolved.clone(), Some(url))
        }
        // Collabora recibe los bytes directamente; solo cambia el contenido.
        (Some(resolved), _) => (resolved.clone(), None),
        (None, _) => (content.to_vec(), None),
    };

    match doc.editor.as_str() {
        "onlyoffice" => onlyoffice_converter::to_pdf(doc, &conversion_content, source_url).await,
        "collabora" => collabora_converter::to_pdf(doc, &conversion_content).await,
        _ => Err("Editor no compatible para conversión".to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/pdf",
    params(
        ("id" = String, Path, description = "Document id")
    ),
    responses(
        (status = 200, description = "PDF bytes"),
        (status = 404, description = "PDF not found")
    ),
    tag = "Documents"
)]
pub async fn get_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    if !document_repo::is_owner(&state.db, &id, &user.sub) && !document_repo::is_shared_with(&state.db, &id, &user.sub) {
        return (StatusCode::FORBIDDEN, "Sin acceso al documento").into_response();
    }
    let path = document_repo::pdf_path(&state.db_path, &id);
    if path.exists() {
        match std::fs::read(&path) {
            Ok(content) => ([(axum::http::header::CONTENT_TYPE, "application/pdf")], content).into_response(),
            Err(_) => (StatusCode::NOT_FOUND, "PDF error").into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "PDF not found. Convert the document first.").into_response()
    }
}

fn pdf_response(content: Vec<u8>, temporary: bool) -> Response {
    let cache = if temporary { "no-store, max-age=0" } else { "private, max-age=60" };
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/pdf"),
            (axum::http::header::CONTENT_DISPOSITION, "inline"),
            (axum::http::header::CACHE_CONTROL, cache),
        ],
        content,
    ).into_response()
}
