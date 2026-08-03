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
use crate::services::{collabora_converter, converter, document_service, onlyoffice_converter};
use crate::repos::document_repo;

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
        ("tab" = Option<String>, Query, description = "Filtro de documentos: `mine` (propios, por defecto) o `shared` (compartidos conmigo).", example = "mine")
    ),
    responses(
        (status = 200, description = "Lista de documentos del usuario según la pestaña.", body = [crate::dto::DocumentResponse]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Documents",
    summary = "Listar documentos",
    description = "Devuelve los documentos del usuario actual. Con `tab=mine` lista los propios; con `tab=shared` \
        los documentos que otros usuarios compartieron con él. Requiere autenticación."
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
        (status = 201, description = "Documento creado correctamente.", body = crate::dto::DocumentResponse),
        (status = 400, description = "Solicitud de documento inválida"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Documents",
    summary = "Crear documento",
    description = "Crea un nuevo documento en blanco (o a partir de una plantilla con `template_id`) \
        y devuelve sus metadatos. Requiere autenticación."
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Metadatos del documento.", body = crate::dto::DocumentResponse),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "Documents",
    summary = "Obtener documento",
    description = "Devuelve los metadatos de un documento por su id."
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Resultado de la eliminación.", body = crate::dto::DeleteResponse),
        (status = 401, description = "Token requerido"),
        (status = 500, description = "Error al eliminar el documento")
    ),
    tag = "Documents",
    summary = "Eliminar documento",
    description = "Elimina el documento y su archivo en disco. Solo el propietario puede eliminarlo. Requiere autenticación."
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Contenido binario original del documento."),
        (status = 404, description = "No encontrado")
    ),
    tag = "Documents",
    summary = "Descargar documento",
    description = "Descarga el archivo binario original del documento. No requiere autenticación."
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Contenido binario del documento."),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "Documents",
    summary = "Obtener contenido",
    description = "Devuelve el contenido binario del documento. Lo usan los editores y el convertidor."
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
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Resultado de la conversión a PDF.", body = ConvertResponse),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Documento no encontrado"),
        (status = 502, description = "Error del servicio de conversión")
    ),
    tag = "Documents",
    summary = "Convertir a PDF",
    description = "Convierte el documento a PDF (vía ONLYOFFICE o Collabora según el editor) y guarda el resultado. \
        Solo el propietario puede convertir. Requiere autenticación."
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

    let pdf_bytes = match converter::content_to_pdf(&state, &doc, &content, &user.sub, &user.name, None).await {
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
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "PDF temporal de previsualización. Resuelve las etiquetas del contenido antes de convertir."),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Documento no encontrado"),
        (status = 502, description = "Error del servicio de conversión")
    ),
    tag = "Documents",
    summary = "Previsualizar documento",
    description = "Genera un PDF temporal del documento para la previsualización en el frontend. \
        Antes de convertir, resuelve las etiquetas `{{ clave }}` por sus valores reales. \
        Requiere ser propietario o tener acceso compartido."
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
    match converter::content_to_pdf(&state, &doc, &content, &user.sub, &user.name, None).await {
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

#[utoipa::path(
    get,
    path = "/api/documents/{id}/pdf",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Bytes del PDF ya convertido."),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "PDF no encontrado (primero debe convertirse el documento)")
    ),
    tag = "Documents",
    summary = "Obtener PDF convertido",
    description = "Devuelve el PDF guardado de un documento previamente convertido. \
        Requiere ser propietario o tener acceso compartido."
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
