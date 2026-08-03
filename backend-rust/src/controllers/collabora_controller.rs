use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use axum::body::Bytes;
use crate::AppState;
use crate::dto::{CheckFileInfo, TokenQuery, JwtClaims};
use crate::helpers::{config, jwt, wopi};
use crate::repos::{document_repo, template_repo};

fn user_or_401(headers: &HeaderMap) -> Result<JwtClaims, Response> {
    jwt::extract_user(headers, &config::jwt_secret())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido".to_string()).into_response())
}

async fn wopi_verify(
    _state: &AppState,
    query: &TokenQuery,
    doc_id: &str,
) -> Result<JwtClaims, Response> {
    let token = query.access_token.as_ref().ok_or_else(|| (StatusCode::UNAUTHORIZED, "No access token").into_response())?;
    let claims = jwt::validate(&config::jwt_secret(), token).ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid token").into_response())?;
    if claims.file_id != doc_id {
        return Err((StatusCode::FORBIDDEN, "Token/file mismatch").into_response());
    }
    Ok(claims)
}

#[utoipa::path(
    get,
    path = "/api/collabora/session/{id}",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
    ),
    responses(
        (status = 200, description = "Sesión de edición de Collabora para incrustar en un iframe.", body = crate::dto::CollaboraSession),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "Collabora",
    summary = "Crear sesión de Collabora",
    description = "Genera la URL del iframe y el token de acceso para editar un documento con Collabora. Requiere autenticación."
)]
pub async fn session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match crate::services::collabora_service::create_session(
        &state.db, &headers, &id, &user.sub, &user.name, &state.collab_browser_prefix,
    ) {
        Ok(session) => Json(session).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/wopi/files/{id}",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Metadatos WOPI del documento (CheckFileInfo).", body = CheckFileInfo),
        (status = 401, description = "Token inválido o ausente"),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "Collabora",
    summary = "WOPI CheckFileInfo (documentos)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para consultar los metadatos de un documento."
)]
pub async fn check_file_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let doc = match document_repo::get_by_id(&state.db, &id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    Json(CheckFileInfo {
        base_file_name: format!("{}.{}", doc.name, doc.ext),
        size: doc.size,
        user_id: claims.sub,
        user_friendly_name: claims.name,
        version: "1.0".to_string(),
        last_modified_time: doc.updated_at,
        user_can_write: true,
        user_can_not_write_relative: false,
        breadcrumb_doc_name: doc.name,
        supports_locks: true,
        supports_get_lock: true,
        supports_update: true,
        supports_extended_lock_length: false,
    }).into_response()
}

#[utoipa::path(
    get,
    path = "/wopi/files/{id}/contents",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Contenido binario del documento."),
        (status = 401, description = "Token inválido o ausente"),
        (status = 404, description = "Documento no encontrado")
    ),
    tag = "Collabora",
    summary = "WOPI GetFile (documentos)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para leer el contenido de un documento."
)]
pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match document_repo::read_file(&state.db_path, &id) {
        Some(content) => {
            let mime = document_repo::get_by_id(&state.db, &id)
                .map(|d| d.mime).unwrap_or_else(|| "application/octet-stream".to_string());
            ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/wopi/files/{id}",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Resultado de la operación WOPI (lock/unlock)."),
        (status = 400, description = "Solicitud WOPI inválida"),
        (status = 401, description = "Token inválido o ausente"),
        (status = 409, description = "Conflicto de bloqueo (X-WOPI-Lock no coincide)")
    ),
    tag = "Collabora",
    summary = "WOPI operaciones de bloqueo (documentos)",
    description = "Endpoint interno del protocolo WOPI para gestionar los bloqueos (LOCK, UNLOCK, REFRESH_LOCK, GET_LOCK) de un documento."
)]
pub async fn file_ops(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let op = match wopi::wopi_override(&headers) {
        Some(op) => op,
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    };
    let lock_val = wopi::wopi_lock_value(&headers);

    match op.as_str() {
        "LOCK" | "REFRESH_LOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &id) {
                Some(existing) if existing != requested => wopi::conflict_response(Some(existing)),
                _ => { wopi::set_lock(&state.wopi_locks, &id, requested.clone()); wopi::lock_response(&requested) }
            }
        }
        "UNLOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &id) {
                Some(existing) if existing == requested => { wopi::clear_lock(&state.wopi_locks, &id); StatusCode::OK.into_response() }
                Some(existing) => wopi::conflict_response(Some(existing)),
                None => StatusCode::OK.into_response(),
            }
        }
        "GET_LOCK" => match wopi::current_lock(&state.wopi_locks, &id) {
            Some(lock) => wopi::lock_response(&lock),
            None => StatusCode::OK.into_response(),
        },
        _ => (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", op)).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/wopi/files/{id}/contents",
    params(
        ("id" = String, Path, description = "Identificador único del documento (UUID).", example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    request_body(content = String, content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Documento guardado. Devuelve `X-WOPI-ItemVersion`."),
        (status = 400, description = "Operación WOPI inválida"),
        (status = 401, description = "Token inválido o ausente"),
        (status = 409, description = "Conflicto de bloqueo (X-WOPI-Lock no coincide)")
    ),
    tag = "Collabora",
    summary = "WOPI PutFile (documentos)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para guardar el contenido editado de un documento."
)]
pub async fn put_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
    body: Bytes,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match wopi::wopi_override(&headers).as_deref() {
        Some("PUT") => {}
        Some(other) => return (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", other)).into_response(),
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    }

    if let Some(current) = wopi::current_lock(&state.wopi_locks, &id) {
        match wopi::wopi_lock_value(&headers) {
            Some(requested) if requested == current => {}
            Some(_) => return wopi::conflict_response(Some(current)),
            None => return wopi::conflict_response(Some(current)),
        }
    }

    let doc = document_repo::get_by_id(&state.db, &id).unwrap();
    document_repo::write_file(&state.db_path, &doc, &body).unwrap_or_default();
    document_repo::update_content(&state.db, &id, body.len() as u64);

    let mut response = StatusCode::OK.into_response();
    response.headers_mut().insert("X-WOPI-ItemVersion",
        HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0")));
    response
}

// ---------------------------------------------------------------------------
// WOPI para plantillas (editar plantillas con Collabora).
// Los locks se almacenan con la clave `tpl:{id}` para no chocar con los de los
// documentos, que usan el id a secas.
// ---------------------------------------------------------------------------

fn template_lock_key(id: &str) -> String {
    format!("tpl:{}", id)
}

#[utoipa::path(
    get,
    path = "/api/collabora/config/template/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")
    ),
    responses(
        (status = 200, description = "Sesión de edición de Collabora para la plantilla.", body = crate::dto::CollaboraSession),
        (status = 401, description = "Token requerido"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Collabora",
    summary = "Crear sesión de Collabora (plantilla)",
    description = "Genera la URL del iframe y el token para editar una plantilla con Collabora. Requiere autenticación."
)]
pub async fn template_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers) { Ok(u) => u, Err(e) => return e };
    match crate::services::collabora_service::create_template_session(
        &state.db,
        &id,
        &user.sub,
        &user.name,
        &state.collab_browser_prefix,
    ) {
        Ok(session) => Json(session).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/wopi/templates/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Metadatos WOPI de la plantilla (CheckFileInfo).", body = CheckFileInfo),
        (status = 401, description = "Token inválido o ausente"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Collabora",
    summary = "WOPI CheckFileInfo (plantillas)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para consultar los metadatos de una plantilla."
)]
pub async fn template_check_file_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let template = match template_repo::get_by_id(&state.db, &id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    Json(CheckFileInfo {
        base_file_name: format!("{}.{}", template.name, template.ext),
        size: template.size,
        user_id: claims.sub,
        user_friendly_name: claims.name,
        version: "1.0".to_string(),
        last_modified_time: template.updated_at,
        user_can_write: true,
        user_can_not_write_relative: false,
        breadcrumb_doc_name: template.name,
        supports_locks: true,
        supports_get_lock: true,
        supports_update: true,
        supports_extended_lock_length: false,
    }).into_response()
}

#[utoipa::path(
    get,
    path = "/wopi/templates/{id}/contents",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Contenido binario de la plantilla."),
        (status = 401, description = "Token inválido o ausente"),
        (status = 404, description = "Plantilla no encontrada")
    ),
    tag = "Collabora",
    summary = "WOPI GetFile (plantillas)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para leer el contenido de una plantilla."
)]
pub async fn template_get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match template_repo::read_file(&state.db_path, &id) {
        Some(content) => {
            let mime = template_repo::get_by_id(&state.db, &id)
                .map(|t| t.mime).unwrap_or_else(|| "application/octet-stream".to_string());
            ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/wopi/templates/{id}",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    responses(
        (status = 200, description = "Resultado de la operación WOPI (lock/unlock)."),
        (status = 400, description = "Solicitud WOPI inválida"),
        (status = 401, description = "Token inválido o ausente"),
        (status = 409, description = "Conflicto de bloqueo (X-WOPI-Lock no coincide)")
    ),
    tag = "Collabora",
    summary = "WOPI operaciones de bloqueo (plantillas)",
    description = "Endpoint interno del protocolo WOPI para gestionar los bloqueos (LOCK, UNLOCK, REFRESH_LOCK, GET_LOCK) de una plantilla."
)]
pub async fn template_file_ops(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    let op = match wopi::wopi_override(&headers) {
        Some(op) => op,
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    };
    let lock_val = wopi::wopi_lock_value(&headers);
    let lock_key = template_lock_key(&id);

    match op.as_str() {
        "LOCK" | "REFRESH_LOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &lock_key) {
                Some(existing) if existing != requested => wopi::conflict_response(Some(existing)),
                _ => { wopi::set_lock(&state.wopi_locks, &lock_key, requested.clone()); wopi::lock_response(&requested) }
            }
        }
        "UNLOCK" => {
            let requested = match &lock_val { Some(l) if !l.is_empty() => l.clone(), _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response() };
            match wopi::current_lock(&state.wopi_locks, &lock_key) {
                Some(existing) if existing == requested => { wopi::clear_lock(&state.wopi_locks, &lock_key); StatusCode::OK.into_response() }
                Some(existing) => wopi::conflict_response(Some(existing)),
                None => StatusCode::OK.into_response(),
            }
        }
        "GET_LOCK" => match wopi::current_lock(&state.wopi_locks, &lock_key) {
            Some(lock) => wopi::lock_response(&lock),
            None => StatusCode::OK.into_response(),
        },
        _ => (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", op)).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/wopi/templates/{id}/contents",
    params(
        ("id" = String, Path, description = "Identificador único de la plantilla (UUID).", example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a"),
        ("access_token" = Option<String>, Query, description = "Token WOPI emitido al crear la sesión.", example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
    ),
    request_body(content = String, content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Plantilla guardada. Devuelve `X-WOPI-ItemVersion`."),
        (status = 400, description = "Operación WOPI inválida"),
        (status = 401, description = "Token inválido o ausente"),
        (status = 409, description = "Conflicto de bloqueo (X-WOPI-Lock no coincide)")
    ),
    tag = "Collabora",
    summary = "WOPI PutFile (plantillas)",
    description = "Endpoint interno del protocolo WOPI usado por Collabora para guardar el contenido editado de una plantilla."
)]
pub async fn template_put_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
    body: Bytes,
) -> Response {
    let _claims = match wopi_verify(&state, &query, &id).await { Ok(c) => c, Err(e) => return e };
    match wopi::wopi_override(&headers).as_deref() {
        Some("PUT") => {}
        Some(other) => return (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", other)).into_response(),
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    }

    let lock_key = template_lock_key(&id);
    if let Some(current) = wopi::current_lock(&state.wopi_locks, &lock_key) {
        match wopi::wopi_lock_value(&headers) {
            Some(requested) if requested == current => {}
            Some(_) => return wopi::conflict_response(Some(current)),
            None => return wopi::conflict_response(Some(current)),
        }
    }

    let template = match template_repo::get_by_id(&state.db, &id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, "Template not found").into_response(),
    };
    template_repo::write_file(&state.db_path, &template, &body).unwrap_or_default();
    template_repo::update_size(&state.db, &id, body.len() as u64);

    let mut response = StatusCode::OK.into_response();
    response.headers_mut().insert("X-WOPI-ItemVersion",
        HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0")));
    response
}
