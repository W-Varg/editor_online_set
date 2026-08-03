use serde::Serialize;
use utoipa::ToSchema;

#[allow(dead_code)]
/// Respuesta del endpoint raíz `/`.
#[derive(Debug, Serialize, ToSchema)]
pub struct RootResponse {
    /// Estado general del servicio: `ok`.
    #[schema(example = "ok")]
    pub status: String,
    /// Identificador del servidor backend.
    #[schema(example = "editor-online-backend")]
    pub server: String,
    /// Versión del backend según Cargo.toml.
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Host que recibió la petición.
    #[schema(example = "localhost:8091")]
    pub host: String,
    /// Fecha y hora de la respuesta (ISO 8601).
    #[schema(example = "2026-08-02T14:00:00Z")]
    pub timestamp: String,
}

#[allow(dead_code)]
/// Respuesta del endpoint de salud `/health`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Estado de salud del servicio: `healthy`.
    #[schema(example = "healthy")]
    pub status: String,
    /// Fecha y hora de la comprobación (ISO 8601).
    #[schema(example = "2026-08-02T14:00:00Z")]
    pub timestamp: String,
}
