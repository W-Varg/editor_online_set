use std::path::PathBuf;
use axum::http::HeaderMap;
use crate::db::DbConn;
use crate::dto::CollaboraSession;
use crate::helpers::{jwt, url};

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

pub fn create_session(
    db: &DbConn,
    db_path: &PathBuf,
    headers: &HeaderMap,
    doc_id: &str,
    user_id: &str,
    user_name: &str,
    browser_prefix: &str,
) -> Result<CollaboraSession, String> {
    let _doc = super::document_service::get_by_id(db, doc_id)
        .ok_or_else(|| "Documento no encontrado".to_string())?;

    let collabora_url = std::env::var("COLLABORA_URL").unwrap_or_else(|_| "http://localhost:8093".to_string());
    let backend_url = url::public_service_url(headers, 8091);

    let ttl_seconds = 86400;
    let token = jwt::create(JWT_SECRET, user_id, user_name, doc_id, ttl_seconds);
    let wopi_src = format!("{}/wopi/files/{}", backend_url, doc_id);
    let encoded_src = url::urlencoding(&wopi_src);

    let collab_path = if browser_prefix.is_empty() {
        format!("{}/loleaflet/dist/loleaflet.html", collabora_url)
    } else {
        format!("{}{}/cool.html", collabora_url, browser_prefix)
    };

    let iframe_url = format!(
        "{}?WOPISrc={}&access_token={}&access_token_ttl={}",
        collab_path, encoded_src, token, ttl_seconds * 1000
    );

    Ok(CollaboraSession { iframe_url, access_token: token })
}
