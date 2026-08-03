use axum::http::HeaderMap;
use crate::db::DbConn;
use crate::dto::CollaboraSession;
use crate::helpers::{config, jwt, url};

/// Construye la sesión de Collabora (iframe + token WOPI) para un archivo
/// identificado por su tipo WOPI (`/wopi/files` para documentos,
/// `/wopi/templates` para plantillas).
fn build_session(
    file_id: &str,
    wopi_prefix: &str,
    user_id: &str,
    user_name: &str,
    browser_prefix: &str,
) -> CollaboraSession {
    let collabora_url = std::env::var("COLLABORA_URL").unwrap_or_else(|_| "http://localhost:8093".to_string());
    let backend_url = config::public_backend_url(8091);

    let ttl_seconds = 86400;
    let token = jwt::create(&config::jwt_secret(), user_id, user_name, file_id, ttl_seconds);
    let wopi_src = format!("{}{}/{}", backend_url, wopi_prefix, file_id);
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

    CollaboraSession { iframe_url, access_token: token }
}

pub fn create_session(
    db: &DbConn,
    _headers: &HeaderMap,
    doc_id: &str,
    user_id: &str,
    user_name: &str,
    browser_prefix: &str,
) -> Result<CollaboraSession, String> {
    let _doc = super::document_service::get_by_id(db, doc_id)
        .ok_or_else(|| "Documento no encontrado".to_string())?;

    Ok(build_session(doc_id, "/wopi/files", user_id, user_name, browser_prefix))
}

/// Sesión de Collabora para *editar una plantilla*: el `WOPISrc` apunta a los
/// endpoints WOPI de plantillas (`/wopi/templates/{id}`) y el JWT usa el id de
/// la plantilla como `file_id`.
pub fn create_template_session(
    db: &DbConn,
    template_id: &str,
    user_id: &str,
    user_name: &str,
    browser_prefix: &str,
) -> Result<CollaboraSession, String> {
    let _template = crate::repos::template_repo::get_by_id(db, template_id)
        .ok_or_else(|| "Plantilla no encontrada".to_string())?;

    Ok(build_session(template_id, "/wopi/templates", user_id, user_name, browser_prefix))
}
