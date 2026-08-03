use std::sync::Arc;
use crate::AppState;
use crate::helpers::config;
use crate::models::Document;
use crate::repos::user_repo;
use crate::services::tag_service;

/// Convierte el contenido de un documento (o plantilla) a PDF, resolviendo antes
/// las etiquetas `{{key}}` con los datos del usuario que lo solicita.
///
/// `fallback_source_url`: URL que ONLYOFFICE debe usar como origen cuando el
/// documento no contiene etiquetas que resolver (para documentos es
/// `/download/{id}`; para plantillas `/api/templates/{id}/content`).
pub async fn content_to_pdf(
    state: &Arc<AppState>,
    doc: &Document,
    content: &[u8],
    user_id: &str,
    session_name: &str,
    fallback_source_url: Option<String>,
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
        // Sin etiquetas: se usa la URL por defecto del tipo de archivo.
        (None, _) => (content.to_vec(), fallback_source_url),
    };

    match doc.editor.as_str() {
        "onlyoffice" => crate::services::onlyoffice_converter::to_pdf(doc, &conversion_content, source_url).await,
        "collabora" => crate::services::collabora_converter::to_pdf(doc, &conversion_content).await,
        _ => Err("Editor no compatible para conversión".to_string()),
    }
}
