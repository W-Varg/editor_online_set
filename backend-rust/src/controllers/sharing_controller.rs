use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use utoipa::ToSchema;
use crate::AppState;
use crate::helpers::{config, jwt};
use crate::repos::user_repo;
use crate::repos::document_repo;
use crate::services::sharing_service;
use crate::dto::{ShareSearchData, ShareSearchResponse, ShareSyncRequest};

fn user_or_401(headers: &HeaderMap) -> Result<crate::dto::JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SharePayload {
    pub user_id: String,
}

#[utoipa::path(
    get,
    path = "/api/users/search",
    params(
        ("q" = String, Query, description = "Texto a buscar por nombre, DNI o nombre de usuario.", example = "ana")
    ),
    responses(
        (status = 200, description = "Usuarios que coinciden con la búsqueda.", body = [crate::dto::UserSearchResult]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing",
    summary = "Buscar usuarios",
    description = "Busca usuarios por nombre, DNI o nombre de usuario. Excluye al usuario autenticado. Requiere autenticación."
)]
pub async fn search_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    Json(user_repo::search(&state.db, query.q.as_deref().unwrap_or(""), &user.sub)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/shares/search",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("q" = Option<String>, Query, description = "DNI, nombre o nombre de usuario a buscar.", example = "ana")
    ),
    responses(
        (status = 200, description = "Usuarios ya compartidos y candidatos encontrados.", body = ShareSearchResponse),
        (status = 401, description = "Token requerido"),
        (status = 403, description = "Solo el propietario puede administrar permisos")
    ),
    tag = "Sharing",
    summary = "Buscar comparticiones del documento",
    description = "Devuelve los usuarios que ya tienen acceso al documento y los candidatos que coinciden \
        con la búsqueda para agregar nuevos. Requiere autenticación."
)]
pub async fn search_document_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    if !document_repo::is_owner(&state.db, &id, &user.sub) {
        return (StatusCode::FORBIDDEN, "Solo el propietario puede administrar los permisos").into_response();
    }
    let (compartidos, encontrados) = user_repo::search_for_document(
        &state.db,
        &id,
        query.q.as_deref().unwrap_or(""),
        &user.sub,
    );
    Json(ShareSearchResponse { data: ShareSearchData { compartidos, encontrados } }).into_response()
}

#[utoipa::path(
    put,
    path = "/api/documents/{id}/shares/sync",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    request_body = ShareSyncRequest,
    responses(
        (status = 200, description = "Comparticiones actualizadas. Devuelve la lista completa.", body = [crate::dto::ShareResponse]),
        (status = 400, description = "Actualización de comparticiones inválida"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing",
    summary = "Sincronizar comparticiones",
    description = "Aplica en lote los cambios de compartición: agrega y/o revoca accesos de una sola vez. \
        Devuelve la lista actualizada de comparticiones. Requiere autenticación."
)]
pub async fn sync(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<ShareSyncRequest>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match sharing_service::sync(&state.db, &id, &user.sub, &payload.add, &payload.remove) {
        Ok(shares) => Json(shares).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, error).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/documents/{id}/shares",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    request_body = SharePayload,
    responses(
        (status = 201, description = "Compartición creada.", body = crate::dto::ShareResponse),
        (status = 400, description = "Solicitud de compartición inválida"),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing",
    summary = "Compartir documento",
    description = "Comparte el documento con otro usuario otorgándole acceso. Requiere ser propietario."
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<SharePayload>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match sharing_service::share(&state.db, &id, &payload.user_id, &user.sub) {
        Ok(share) => (StatusCode::CREATED, Json(share)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/documents/{id}/shares",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Comparticiones actuales del documento.", body = [crate::dto::ShareResponse]),
        (status = 401, description = "Token requerido")
    ),
    tag = "Sharing",
    summary = "Listar comparticiones",
    description = "Devuelve la lista de usuarios con los que está compartido el documento. Requiere autenticación."
)]
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let _user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    Json(sharing_service::list(&state.db, &id)).into_response()
}

#[utoipa::path(
    delete,
    path = "/api/documents/{id}/shares/{user_id}",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("user_id" = String, Path, description = "Identificador del usuario al que se revocará el acceso.", example = "2")
    ),
    responses(
        (status = 200, description = "Acceso revocado."),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Compartición no encontrada")
    ),
    tag = "Sharing",
    summary = "Revocar compartición",
    description = "Elimina el acceso de un usuario a un documento. Requiere autenticación."
)]
pub async fn remove(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, user_id)): Path<(String, String)>,
) -> Response {
    let _user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match sharing_service::remove(&state.db, &id, &user_id) {
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}
