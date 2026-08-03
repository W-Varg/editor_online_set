mod controllers;
mod db;
mod dto;
mod helpers;
mod models;
mod openapi;
mod repos;
mod services;
pub mod templates;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Extension, Path},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::{Config, Url};

use crate::dto::system::{HealthResponse, RootResponse};
use db::DbConn;
use openapi::ApiDoc;

/// Contenido de un documento con las etiquetas ya resueltas, registrado con un
/// token de un solo uso para que el convertidor de ONLYOFFICE pueda descargarlo
/// por URL (el converter no recibe bytes, fetchea `fileUrl`).
pub(crate) struct PreviewSource {
    content: Vec<u8>,
    mime: String,
    expires_at: std::time::Instant,
}

pub struct AppState {
    pub db: DbConn,
    pub db_path: std::path::PathBuf,
    pub jwt_secret: String,
    pub collab_browser_prefix: String,
    pub backend_url: String,
    pub wopi_locks: Arc<Mutex<HashMap<String, String>>>,
    pub(crate) preview_sources: Arc<Mutex<HashMap<String, PreviewSource>>>,
}

async fn discover_collab_prefix(base_url: &str) -> String {
    let discovery_url = format!("{}/hosting/discovery", base_url);
    match reqwest::get(&discovery_url).await {
        Ok(resp) => {
            if let Ok(body) = resp.text().await {
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
            tracing::warn!(
                "Failed to fetch Collabora discovery ({}), using loleaflet fallback",
                e
            );
            String::new()
        }
    }
}

// ---- Root / Health / API Docs ----

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service status", body = crate::dto::system::RootResponse)
    )
)]
async fn root_handler(headers: HeaderMap) -> Json<RootResponse> {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    Json(RootResponse {
        status: "ok".to_string(),
        server: "editor-online-backend".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        host: host.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health", body = crate::dto::system::HealthResponse)
    )
)]
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Sirve la interfaz Swagger UI y sus recursos en `/api`, sin redirecciones.
async fn swagger_ui(
    path: Option<Path<String>>,
    Extension(config): Extension<Arc<Config<'static>>>,
) -> Response {
    let tail = path.map(|p| p.0).unwrap_or_default();
    match utoipa_swagger_ui::serve(&tail, config) {
        Ok(Some(file)) => (
            [(axum::http::header::CONTENT_TYPE, file.content_type.as_str())],
            file.bytes.to_vec(),
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response(),
    }
}

/// Sirve el documento OpenAPI en formato JSON puro, para la propia UI de Swagger
/// y para herramientas externas (clientes HTTP, generadores de código, etc.).
#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "Documento OpenAPI 3.0 en JSON")
    ),
    tag = "System"
)]
async fn serve_api_docs() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[tokio::main]
async fn main() {
    // Load backend-rust/.env independently of the directory used to start the process.
    dotenvy::from_filename(format!("{}/.env", env!("CARGO_MANIFEST_DIR"))).ok();
    // Keep support for a repository-level .env when running from backend-rust/.
    dotenvy::from_filename("../.env").ok();
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8091".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "my-secret-key".to_string());
    let backend_url = std::env::var("BACKEND_URL")
        .unwrap_or_else(|_| "http://host.docker.internal:8091".to_string());
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let public_dir = std::env::var("PUBLIC_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("public"));
    let public_dir = if public_dir.is_absolute() {
        public_dir
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(public_dir)
    };

    tracing::info!(
        public_dir = %public_dir.display(),
        exists = public_dir.is_dir(),
        "Serving public files"
    );
    let plugins_dir = public_dir.join("plugins");
    tracing::info!(
        plugins_dir = %plugins_dir.display(),
        exists = plugins_dir.is_dir(),
        "Serving plugin files"
    );

    let (db, db_path) = db::open_connection(&data_dir);
    db::migrations::run_migrations(&db.lock().unwrap());
    repos::user_repo::seed(&db);

    let collab_url =
        std::env::var("COLLABORA_URL").unwrap_or_else(|_| "http://localhost:8093".to_string());
    let collab_browser_prefix = discover_collab_prefix(&collab_url).await;

    let state = AppState {
        db,
        db_path,
        jwt_secret,
        collab_browser_prefix,
        backend_url,
        wopi_locks: Arc::new(Mutex::new(HashMap::new())),
        preview_sources: Arc::new(Mutex::new(HashMap::new())),
    };

    let swagger_config = Arc::new(Config::new(vec![Url::new(
        "Editor Online Backend",
        "/api-docs/openapi.json",
    )]));

    let swagger_router: Router<Arc<AppState>> = Router::new()
        .route("/api", get(swagger_ui))
        .route("/api/{*tail}", get(swagger_ui))
        .route("/api-docs/openapi.json", get(serve_api_docs))
        .layer(Extension(swagger_config));

    let scalar_router: Router<Arc<AppState>> =
        Scalar::with_url("/scalar", ApiDoc::openapi()).into();

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/auth/login", post(controllers::auth_controller::login))
        .route("/api/users", get(controllers::auth_controller::list_users))
        .route(
            "/api/documents",
            get(controllers::document_controller::list)
                .post(controllers::document_controller::create),
        )
        .route(
            "/api/documents/{id}",
            get(controllers::document_controller::get)
                .delete(controllers::document_controller::delete),
        )
        .route(
            "/api/documents/{id}/content",
            get(controllers::document_controller::content),
        )
        .route(
            "/api/documents/{id}/preview",
            get(controllers::document_controller::preview),
        )
        .route(
            "/api/documents/{id}/convert",
            post(controllers::document_controller::convert_to_pdf),
        )
        .route(
            "/api/documents/{id}/pdf",
            get(controllers::document_controller::get_pdf),
        )
        .route(
            "/api/templates",
            get(controllers::template_controller::list)
                .post(controllers::template_controller::create),
        )
        .route(
            "/api/templates/{id}",
            get(controllers::template_controller::get)
                .put(controllers::template_controller::rename)
                .delete(controllers::template_controller::delete),
        )
        .route(
            "/api/templates/{id}/content",
            get(controllers::template_controller::content),
        )
        .route(
            "/api/templates/{id}/preview",
            get(controllers::template_controller::preview),
        )
        .route(
            "/api/onlyoffice/config/template/{id}",
            get(controllers::template_controller::config),
        )
        .route(
            "/callback/template/{id}",
            post(controllers::template_controller::callback),
        )
        .route(
            "/api/users/search",
            get(controllers::sharing_controller::search_users),
        )
        .route(
            "/api/documents/{id}/shares",
            get(controllers::sharing_controller::list)
                .post(controllers::sharing_controller::create),
        )
        .route(
            "/api/documents/{id}/shares/search",
            get(controllers::sharing_controller::search_document_users),
        )
        .route(
            "/api/documents/{id}/shares/sync",
            axum::routing::put(controllers::sharing_controller::sync),
        )
        .route(
            "/api/documents/{id}/shares/{user_id}",
            delete(controllers::sharing_controller::remove),
        )
        .route("/api/tags", get(controllers::tag_controller::list_tags))
        .route(
            "/api/preview-source/{token}",
            get(controllers::tag_controller::preview_source),
        )
        .route(
            "/api/collabora/session/{id}",
            get(controllers::collabora_controller::session),
        )
        .route(
            "/api/collabora/config/template/{id}",
            get(controllers::collabora_controller::template_session),
        )
        .route(
            "/wopi/files/{id}",
            get(controllers::collabora_controller::check_file_info)
                .post(controllers::collabora_controller::file_ops),
        )
        .route(
            "/wopi/files/{id}/contents",
            get(controllers::collabora_controller::get_file)
                .post(controllers::collabora_controller::put_file),
        )
        .route(
            "/wopi/templates/{id}",
            get(controllers::collabora_controller::template_check_file_info)
                .post(controllers::collabora_controller::template_file_ops),
        )
        .route(
            "/wopi/templates/{id}/contents",
            get(controllers::collabora_controller::template_get_file)
                .post(controllers::collabora_controller::template_put_file),
        )
        .route(
            "/api/onlyoffice/config/{id}",
            get(controllers::onlyoffice_controller::config),
        )
        .route(
            "/download/{id}",
            get(controllers::document_controller::download),
        )
        .route(
            "/callback/onlyoffice/{id}",
            post(controllers::onlyoffice_controller::callback),
        )
        .merge(swagger_router)
        .merge(scalar_router)
        .nest_service("/plugins", ServeDir::new(plugins_dir))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Backend starting on {}", addr);

    println!("🚀 Servidor Rust + Axum escuchando en el puerto {port}");
    println!("📘 Documentación Swagger disponible en http://localhost:{port}/api/index.html");
    println!("📖 Documentación Scalar disponible en http://localhost:{port}/scalar");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
