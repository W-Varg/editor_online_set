use serde::Serialize;
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Serialize, ToSchema)]
pub struct RootResponse {
    pub status: String,
    pub server: String,
    pub version: String,
    pub host: String,
    pub timestamp: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
}
