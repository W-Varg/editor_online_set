mod models;
mod storage;
mod templates;

use std::collections::HashMap;
use std::sync::Arc;

async fn discover_collab_prefix(base_url: &str) -> String {
    let discovery_url = format!("{}/hosting/discovery", base_url);
    match reqwest::get(&discovery_url).await {
        Ok(resp) => {
            if let Ok(body) = resp.text().await {
                // Look for pattern: /browser/{ID}/cool.html in urlsrc attribute
                if let Some(pos) = body.find("urlsrc=\"") {
                    let after = &body[pos + 8..];
                    if let Some(start) = after.find("/browser/") {
                        let from_start = &after[start..];
                        if let Some(end) = from_start.find("/cool.html") {
                            let prefix = &from_start[..end];
                            tracing::info!("Discovered Collabora browser prefix: {}", prefix);
                            return prefix.to_string();
                        }
                    }
                }
            }
            tracing::warn!("Could not parse Collabora discovery, using loleaflet fallback");
            String::new()
        }
        Err(e) => {
            tracing::warn!("Failed to fetch Collabora discovery ({}), using loleaflet fallback", e);
            String::new()
        }
    }
}

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use models::*;
use storage::Storage;

struct AppState {
    storage: Storage,
    jwt_secret: String,
    collab_browser_prefix: String,
    backend_url: String,
    wopi_locks: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

#[derive(Deserialize)]
struct TokenQuery {
    access_token: Option<String>,
}

fn create_jwt(secret: &str, user_id: &str, user_name: &str, file_id: &str, ttl_seconds: i64) -> String {
    let claims = JwtClaims {
        sub: user_id.to_string(),
        file_id: file_id.to_string(),
        name: user_name.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds)).timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
}

fn validate_jwt(secret: &str, token: &str) -> Option<JwtClaims> {
    decode::<JwtClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256))
        .ok().map(|d| d.claims)
}

fn extract_user(headers: &HeaderMap, secret: &str) -> Option<JwtClaims> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    validate_jwt(secret, token)
}

fn urlencoding(s: &str) -> String {
    s.as_bytes().iter().map(|&c| {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (c as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", c),
        }
    }).collect()
}

fn host_without_port(headers: &HeaderMap) -> Option<String> {
    let host = headers.get("host")?.to_str().ok()?;
    Some(host.split(':').next().unwrap_or(host).to_string())
}

fn public_service_url(headers: &HeaderMap, port: u16) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");
    let host = host_without_port(headers).unwrap_or_else(|| "localhost".to_string());
    format!("{}://{}:{}", scheme, host, port)
}

fn onlyoffice_document_key(id: &str, content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(b":");
    hasher.update(content);
    format!("{}-{:x}", id, hasher.finalize())
}

fn wopi_override(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-wopi-override")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_uppercase())
}

fn wopi_lock_value(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-wopi-lock")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

fn current_wopi_lock(state: &AppState, id: &str) -> Option<String> {
    state.wopi_locks.lock().ok().and_then(|locks| locks.get(id).cloned())
}

fn set_wopi_lock(state: &AppState, id: &str, lock: String) {
    if let Ok(mut locks) = state.wopi_locks.lock() {
        locks.insert(id.to_string(), lock);
    }
}

fn clear_wopi_lock(state: &AppState, id: &str) {
    if let Ok(mut locks) = state.wopi_locks.lock() {
        locks.remove(id);
    }
}

fn wopi_lock_response(lock: &str) -> Response {
    let mut response = StatusCode::OK.into_response();
    response
        .headers_mut()
        .insert("X-WOPI-Lock", HeaderValue::from_str(lock).unwrap_or_else(|_| HeaderValue::from_static("")));
    response
}

fn wopi_conflict_response(lock: Option<String>) -> Response {
    let mut response = (StatusCode::CONFLICT, "Lock mismatch").into_response();
    if let Some(lock) = lock {
        response.headers_mut().insert(
            "X-WOPI-Lock",
            HeaderValue::from_str(&lock).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
    }
    response
}

// ---- Auth ----

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let user = match state.storage.authenticate(&payload.username, &payload.password) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Usuario o contraseña incorrectos").into_response(),
    };
    let token = create_jwt(&state.jwt_secret, &user.id, &user.name, "", 3600);
    Json(AuthResponse { token, user }).into_response()
}

// ---- Document CRUD (autenticado) ----

fn user_or_401(headers: &HeaderMap, secret: &str) -> Result<JwtClaims, Response> {
    extract_user(headers, secret).ok_or_else(|| (StatusCode::UNAUTHORIZED, "Token requerido").into_response())
}

async fn list_docs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let _user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    Json(state.storage.list_documents()).into_response()
}

async fn create_doc(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateDocument>,
) -> Response {
    let _user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    match state.storage.create_document(&payload.name, &payload.ext, &payload.editor) {
        Ok(doc) => (StatusCode::CREATED, Json(doc)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn get_doc(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.storage.get_document(&id) {
        Some(doc) => Json(doc).into_response(),
        None => (StatusCode::NOT_FOUND, "Document not found").into_response(),
    }
}

async fn delete_doc(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let _user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    match state.storage.delete_document(&id) {
        Ok(_) => Json(serde_json::json!({"deleted": true})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn convert_to_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let _user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    let doc = match state.storage.get_document(&id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };

    if state.storage.pdf_exists(&id) {
        let pdf_url = format!("{}/api/documents/{}/pdf", public_service_url(&headers, 8091), id);
        return Json(ConvertResponse {
            pdf_id: format!("{}.pdf", id),
            pdf_url,
            status: "already_converted".to_string(),
        }).into_response();
    }

    let text = format!(
        "Documento: {}\nTipo: {}\nEstado: {}\nCreado: {}",
        doc.name, doc.ext, doc.status, doc.created_at
    );
    let pdf_bytes = templates::generate_pdf(&format!("{}.{}", doc.name, doc.ext), &text);

    match state.storage.save_pdf(&id, &pdf_bytes) {
        Ok(_) => {
            let pdf_url = format!("{}/api/documents/{}/pdf", public_service_url(&headers, 8091), id);
            Json(ConvertResponse {
                pdf_id: format!("{}.pdf", id),
                pdf_url,
                status: "converted".to_string(),
            }).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn get_pdf(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.storage.get_pdf_content(&id) {
        Some(content) => {
            ([(axum::http::header::CONTENT_TYPE, "application/pdf")], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "PDF not found. Convert the document first.").into_response(),
    }
}

// ---- Downloads (sin auth, usado por editores) ----

async fn doc_content(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.storage.get_document_content(&id) {
        Some(content) => {
            let mime = state.storage.get_document(&id)
                .map(|d| d.mime)
                .unwrap_or_else(|| "application/octet-stream".to_string());
            ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Document not found").into_response(),
    }
}

// ---- Collabora WOPI ----

async fn collab_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    let _doc = match state.storage.get_document(&id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };
    let token = create_jwt(&state.jwt_secret, &user.sub, &user.name, &id, 28800);
    let wopi_src = format!("{}/wopi/files/{}", state.backend_url, id);
    let encoded_src = urlencoding(&wopi_src);
    let public_collab_url = std::env::var("PUBLIC_COLLABORA_URL")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| public_service_url(&headers, 8093));
    let collab_path = if state.collab_browser_prefix.is_empty() {
        format!("{}/loleaflet/dist/loleaflet.html", public_collab_url)
    } else {
        format!("{}{}/cool.html", public_collab_url, state.collab_browser_prefix)
    };
    let iframe_url = format!(
        "{}?WOPISrc={}&access_token={}&access_token_ttl=28800",
        collab_path, encoded_src, token
    );
    Json(CollaboraSession { iframe_url, access_token: token }).into_response()
}

async fn wopi_check_file_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let token = match &query.access_token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "No access token").into_response(),
    };
    let claims = match validate_jwt(&state.jwt_secret, token) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    if claims.file_id != id {
        return (StatusCode::FORBIDDEN, "Token/file mismatch").into_response();
    }
    let doc = match state.storage.get_document(&id) {
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

async fn wopi_get_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let token = match &query.access_token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "No access token").into_response(),
    };
    let claims = match validate_jwt(&state.jwt_secret, token) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    if claims.file_id != id {
        return (StatusCode::FORBIDDEN, "Token/file mismatch").into_response();
    }
    match state.storage.get_document_content(&id) {
        Some(content) => {
            let mime = state.storage.get_document(&id)
                .map(|d| d.mime)
                .unwrap_or_else(|| "application/octet-stream".to_string());
            ([(axum::http::header::CONTENT_TYPE, mime.as_str())], content).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

async fn wopi_file_ops(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
) -> Response {
    let token = match &query.access_token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "No access token").into_response(),
    };
    let claims = match validate_jwt(&state.jwt_secret, token) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    if claims.file_id != id {
        return (StatusCode::FORBIDDEN, "Token/file mismatch").into_response();
    }

    let op = match wopi_override(&headers) {
        Some(op) => op,
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    };

    match op.as_str() {
        "LOCK" => {
            let requested = match wopi_lock_value(&headers) {
                Some(lock) if !lock.is_empty() => lock,
                _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response(),
            };
            let current = current_wopi_lock(&state, &id);
            match current {
                Some(existing) if existing != requested => wopi_conflict_response(Some(existing)),
                _ => {
                    set_wopi_lock(&state, &id, requested.clone());
                    wopi_lock_response(&requested)
                }
            }
        }
        "REFRESH_LOCK" => {
            let requested = match wopi_lock_value(&headers) {
                Some(lock) if !lock.is_empty() => lock,
                _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response(),
            };
            match current_wopi_lock(&state, &id) {
                Some(existing) if existing != requested => wopi_conflict_response(Some(existing)),
                Some(_) => {
                    set_wopi_lock(&state, &id, requested.clone());
                    wopi_lock_response(&requested)
                }
                None => {
                    set_wopi_lock(&state, &id, requested.clone());
                    wopi_lock_response(&requested)
                }
            }
        }
        "UNLOCK" => {
            let requested = match wopi_lock_value(&headers) {
                Some(lock) if !lock.is_empty() => lock,
                _ => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Lock").into_response(),
            };
            match current_wopi_lock(&state, &id) {
                Some(existing) if existing == requested => {
                    clear_wopi_lock(&state, &id);
                    StatusCode::OK.into_response()
                }
                Some(existing) => wopi_conflict_response(Some(existing)),
                None => StatusCode::OK.into_response(),
            }
        }
        "GET_LOCK" => match current_wopi_lock(&state, &id) {
            Some(lock) => wopi_lock_response(&lock),
            None => StatusCode::OK.into_response(),
        },
        _ => (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", op)).into_response(),
    }
}

async fn wopi_put_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TokenQuery>,
    body: axum::body::Bytes,
) -> Response {
    let token = match &query.access_token {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "No access token").into_response(),
    };
    let claims = match validate_jwt(&state.jwt_secret, token) {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    if claims.file_id != id {
        return (StatusCode::FORBIDDEN, "Token/file mismatch").into_response();
    }
    match wopi_override(&headers).as_deref() {
        Some("PUT") => {}
        Some(other) => return (StatusCode::BAD_REQUEST, format!("Unsupported WOPI operation: {}", other)).into_response(),
        None => return (StatusCode::BAD_REQUEST, "Missing X-WOPI-Override").into_response(),
    }

    if let Some(current_lock) = current_wopi_lock(&state, &id) {
        match wopi_lock_value(&headers) {
            Some(requested) if requested == current_lock => {}
            Some(requested) => {
                tracing::warn!(
                    "Rejecting PutFile for {} because the lock does not match (current={}, requested={})",
                    id, current_lock, requested
                );
                return wopi_conflict_response(Some(current_lock));
            }
            None => {
                tracing::warn!("Rejecting PutFile for {} because no X-WOPI-Lock header was provided", id);
                return wopi_conflict_response(Some(current_lock));
            }
        }
    }

    match state.storage.update_document_content(&id, &body) {
        Ok(_) => {
            let mut response = StatusCode::OK.into_response();
            response.headers_mut().insert(
                "X-WOPI-ItemVersion",
                HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
            response
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

// ---- ONLYOFFICE ----

async fn oo_download(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let doc = match state.storage.get_document(&id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    let content = match state.storage.get_document_content(&id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    ([(axum::http::header::CONTENT_TYPE, doc.mime.as_str())], content).into_response()
}

async fn oo_callback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let status = payload.get("status").and_then(|s| s.as_i64()).unwrap_or(0);
    let url = payload.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
    if status == 2 || status == 3 {
        if let Some(download_url) = url {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build().unwrap_or_default();
            match client.get(&download_url).send().await {
                Ok(resp) => {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Err(e) = state.storage.update_document_content(&id, &bytes) {
                            tracing::error!("Callback save error for doc {}: {}", id, e);
                        } else {
                            tracing::info!("Callback saved document {} ({} bytes)", id, bytes.len());
                        }
                    }
                }
                Err(e) => tracing::error!("Callback: failed to download from URL for doc {}: {}", id, e),
            }
        }
    }
    Json(serde_json::json!({"error": 0}))
}

async fn oo_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match user_or_401(&headers, &state.jwt_secret) {
        Ok(u) => u,
        Err(e) => return e,
    };
    let doc = match state.storage.get_document(&id) {
        Some(d) => d,
        None => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
    };
    let content = match state.storage.get_document_content(&id) {
        Some(bytes) => bytes,
        None => return (StatusCode::NOT_FOUND, "Document content not found").into_response(),
    };
    let backend_url = state.backend_url.clone();

    let document_type = match doc.ext.as_str() {
        "docx" | "doc" => "word".to_string(),
        "xlsx" | "xls" => "cell".to_string(),
        "pptx" | "ppt" => "slide".to_string(),
        _ => "word".to_string(),
    };

    let config = OnlyOfficeConfig {
        document_type,
        document: OnlyOfficeDocument {
            file_type: doc.ext.clone(),
            key: onlyoffice_document_key(&id, &content),
            title: format!("{}.{}", doc.name, doc.ext),
            url: format!("{}/download/{}", backend_url, id),
            permissions: OnlyOfficePermissions {
                edit: true,
                comment: false,
                download: true,
                print: true,
                review: false,
            },
        },
        editor_config: OnlyOfficeEditorConfig {
            callback_url: format!("{}/callback/onlyoffice/{}", backend_url, id),
            lang: "es-ES".to_string(),
            mode: "edit".to_string(),
            customization: OnlyOfficeCustomization { autosave: true, forcesave: true },
            user: OnlyOfficeUser { id: user.sub, name: user.name },
        },
        token: None,
    };

    let config_json = serde_json::to_value(&config).unwrap();
    let jwt_token = encode(
        &Header::default(),
        &config_json,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).unwrap_or_default();

    Json(OnlyOfficeConfig { token: Some(jwt_token), ..config }).into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8091".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "my-secret-key".to_string());
    let backend_url = std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://host.docker.internal:8091".to_string());
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    let storage = Storage::new(&data_dir);
    storage.init();

    // Discover Collabora browser prefix
    let collab_url_internal = std::env::var("COLLABORA_URL").unwrap_or_else(|_| "http://localhost:8093".to_string());
    let collab_browser_prefix = discover_collab_prefix(&collab_url_internal).await;

    let state = Arc::new(AppState {
        storage,
        jwt_secret,
        collab_browser_prefix,
        backend_url,
        wopi_locks: Arc::new(std::sync::Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/documents", get(list_docs).post(create_doc))
        .route("/api/documents/:id", get(get_doc).delete(delete_doc))
        .route("/api/documents/:id/content", get(doc_content))
        .route("/api/documents/:id/convert", post(convert_to_pdf))
        .route("/api/documents/:id/pdf", get(get_pdf))
        .route("/api/collabora/session/:id", get(collab_session))
        .route("/api/onlyoffice/config/:id", get(oo_config))
        .route("/wopi/files/:id", get(wopi_check_file_info).post(wopi_file_ops))
        .route("/wopi/files/:id/contents", get(wopi_get_file).post(wopi_put_file))
        .route("/download/:id", get(oo_download))
        .route("/callback/onlyoffice/:id", post(oo_callback))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Backend starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
