use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use crate::AppState;
use crate::helpers::{config, jwt};
use crate::services::tag_service;

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[utoipa::path(
    get,
    path = "/api/tags",
    responses(
        (status = 200, description = "Catálogo de etiquetas disponibles para insertar en los documentos.", body = [crate::dto::TagDefinition]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Tags",
    summary = "Listar etiquetas",
    description = "Devuelve el catálogo de etiquetas dinámicas (`{{ clave }}`) que el plugin de etiquetas puede \
        insertar en el contenido. El backend las reemplaza por sus valores al previsualizar. Requiere autenticación."
)]
pub async fn list_tags(State(_state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(e) = user_or_401(&headers) {
        return e;
    }
    Json(tag_service::list()).into_response()
}

#[utoipa::path(
    get,
    path = "/api/preview-source/{token}",
    params(
        ("token" = String, Path, description = "Token de un solo uso del contenido del documento con las etiquetas ya resueltas.", example = "a1b2c3d4...")
    ),
    responses(
        (status = 200, description = "Contenido del documento con las etiquetas resueltas."),
        (status = 404, description = "Token no válido o expirado")
    ),
    tag = "Tags",
    summary = "Obtener fuente de previsualización",
    description = "Devuelve el contenido resuelto (etiquetas reemplazadas por valores) asociado a un token \
        de un solo uso. Lo consume el convertidor de ONLYOFFICE; el token expira en 60 segundos."
)]
pub async fn preview_source(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Response {
    let mut store = state.preview_sources.lock().unwrap();
    match store.remove(&token) {
        Some(entry) => {
            if entry.expires_at < std::time::Instant::now() {
                return (StatusCode::NOT_FOUND, "Token expirado").into_response();
            }
            (
                [
                    (axum::http::header::CONTENT_TYPE, entry.mime.as_str()),
                    (axum::http::header::CACHE_CONTROL, "no-store"),
                ],
                entry.content,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Token no válido").into_response(),
    }
}
