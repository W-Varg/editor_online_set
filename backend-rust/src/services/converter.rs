use std::sync::Arc;
use crate::AppState;
use crate::helpers::config;
use crate::models::Document;
use crate::repos::user_repo;
use crate::services::{header_footer_service, tag_service};

/// Convierte el contenido de un documento (o plantilla) a PDF, resolviendo antes
/// las etiquetas `{{key}}` con los datos del usuario que lo solicita.
///
/// `fallback_source_url`: URL que ONLYOFFICE debe usar como origen cuando el
/// documento no contiene etiquetas que resolver (para documentos es
/// `/download/{id}`; para plantillas `/api/templates/{id}/content`).
///
/// `header_footer`: modo de encabezado/pie (`preserve` respeta el existente;
/// `replace` inyecta el encabezado/pie de los archivos editables del sistema
/// ANTES de resolver las etiquetas, para que las `{{key}}` internas también
/// se resuelvan).
pub async fn content_to_pdf(
    state: &Arc<AppState>,
    doc: &Document,
    content: &[u8],
    user_id: &str,
    session_name: &str,
    fallback_source_url: Option<String>,
    header_footer: header_footer_service::HeaderFooterMode,
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
    // 1º: inyectar el encabezado/pie (solo si `replace`); puede contener tags.
    let injected = header_footer_service::inject(content, doc, header_footer, &state.db_path);
    let base = injected.as_deref().unwrap_or(content);
    // 2º: resolver las etiquetas sobre el contenido (incluidas las del header/pie).
    let resolved = user.and_then(|u| tag_service::resolve(base, &u, &doc.ext));
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
        // Sin etiquetas: se usa el contenido ya inyectado (si lo hubo) y la URL
        // por defecto del tipo de archivo (solo la usa ONLYOFFICE).
        (None, "onlyoffice") => (base.to_vec(), fallback_source_url),
        (None, _) => (base.to_vec(), None),
    };

    match doc.editor.as_str() {
        "onlyoffice" => crate::services::onlyoffice_converter::to_pdf(doc, &conversion_content, source_url).await,
        "collabora" => crate::services::collabora_converter::to_pdf(doc, &conversion_content).await,
        _ => Err("Editor no compatible para conversión".to_string()),
    }
}
