use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Parámetros de consulta usados por los endpoints WOPI de Collabora.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TokenQuery {
    /// Token JWT firmado por el backend que autoriza la operación WOPI.
    #[param(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: Option<String>,
}

/// Sesión de edición de Collabora para incrustar en el frontend.
#[derive(Debug, Serialize, ToSchema)]
pub struct CollaboraSession {
    /// URL del iframe de Collabora (cool.html) con el token y el archivo.
    #[schema(example = "http://localhost:8093/browser/ef1d73f0a1cc44830d1be7b42b47afc0/cool.html?access_token=...")]
    pub iframe_url: String,
    /// Token JWT de acceso que debe enviarse a Collabora.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
}

/// Reclamaciones contenidas en el JWT interno del backend.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JwtClaims {
    /// ID del usuario autenticado (subject).
    #[schema(example = "1")]
    pub sub: String,
    /// ID del documento o plantilla al que se refiere el token.
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")]
    pub file_id: String,
    /// Nombre visible del usuario.
    #[schema(example = "Juan Pérez")]
    pub name: String,
    /// Timestamp de expiración del token (segundos Unix).
    #[schema(example = 1754160000)]
    pub exp: usize,
}

/// Respuesta WOPI `CheckFileInfo` con metadatos del archivo para Collabora.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckFileInfo {
    /// Nombre base del archivo con su extensión.
    #[serde(rename = "BaseFileName")]
    #[schema(example = "Contrato de servicios.docx")]
    pub base_file_name: String,
    /// Tamaño del archivo en bytes.
    #[serde(rename = "Size")]
    #[schema(example = 24576)]
    pub size: u64,
    /// ID del usuario en Collabora.
    #[serde(rename = "UserId")]
    #[schema(example = "1")]
    pub user_id: String,
    /// Nombre visible del usuario en Collabora.
    #[serde(rename = "UserFriendlyName")]
    #[schema(example = "Juan Pérez")]
    pub user_friendly_name: String,
    /// Versión del documento comunicada a Collabora.
    #[serde(rename = "Version")]
    #[schema(example = "1.0")]
    pub version: String,
    /// Fecha de última modificación (ISO 8601).
    #[serde(rename = "LastModifiedTime")]
    #[schema(example = "2026-08-02T18:30:00Z")]
    pub last_modified_time: String,
    /// Indica si el usuario puede editar el documento.
    #[serde(rename = "UserCanWrite")]
    #[schema(example = true)]
    pub user_can_write: bool,
    /// Indica si el usuario NO puede crear archivos relativos.
    #[serde(rename = "UserCanNotWriteRelative")]
    #[schema(example = false)]
    pub user_can_not_write_relative: bool,
    /// Nombre mostrado en la barra de navegación de Collabora.
    #[serde(rename = "BreadcrumbDocName")]
    #[schema(example = "Contrato de servicios")]
    pub breadcrumb_doc_name: String,
    /// Habilita el soporte de bloqueos (locks) WOPI.
    #[serde(rename = "SupportsLocks")]
    #[schema(example = true)]
    pub supports_locks: bool,
    /// Habilita la operación `GET_LOCK`.
    #[serde(rename = "SupportsGetLock")]
    #[schema(example = true)]
    pub supports_get_lock: bool,
    /// Habilita la operación `PUT` de guardado.
    #[serde(rename = "SupportsUpdate")]
    #[schema(example = true)]
    pub supports_update: bool,
    /// Habilita bloqueos con duración extendida.
    #[serde(rename = "SupportsExtendedLockLength")]
    #[schema(example = false)]
    pub supports_extended_lock_length: bool,
}
