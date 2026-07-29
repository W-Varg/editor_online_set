use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use crate::AppState;
use crate::dto::{CreateDocument, ConvertResponse};
use crate::helpers::{jwt, url};
use crate::services::document_service;
use crate::repos::document_repo;

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, JWT_SECRET)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[derive(Deserialize)]
pub struct DocQuery {
    pub tab: Option<String>,
}

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

pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match document_service::get_by_id(&state.db, &id) {
        Some(doc) => Json(doc).into_response(),
        None => (StatusCode::NOT_FOUND, "Document not found").into_response(),
    }
}

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

pub async fn convert_to_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let _user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    let doc = match document_service::get_by_id(&state.db, &id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };

    if document_repo::pdf_path(&state.db_path, &id).exists() {
        let pdf_url = format!("{}/api/documents/{}/pdf", url::public_service_url(&headers, 8091), id);
        return Json(ConvertResponse { pdf_id: format!("{}.pdf", id), pdf_url, status: "already_converted".to_string() }).into_response();
    }

    let text = format!("Documento: {}\nTipo: {}\nEstado: {}\nCreado: {}", doc.name, doc.ext, doc.status, doc.created_at);
    let pdf_bytes = crate::templates::generate_pdf(&format!("{}.{}", doc.name, doc.ext), &text);

    match std::fs::write(document_repo::pdf_path(&state.db_path, &id), &pdf_bytes) {
        Ok(_) => {
            let now = chrono::Utc::now().to_rfc3339();
            let conn = state.db.lock().unwrap();
            conn.execute(
                "UPDATE documents SET status = 'final', updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id],
            ).unwrap_or_default();
            let pdf_url = format!("{}/api/documents/{}/pdf", url::public_service_url(&headers, 8091), id);
            Json(ConvertResponse { pdf_id: format!("{}.pdf", id), pdf_url, status: "converted".to_string() }).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_pdf(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
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
