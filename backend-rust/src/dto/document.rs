use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDocument {
    pub name: String,
    pub ext: String,
    pub editor: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentResponse {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub editor: String,
    pub size: u64,
    pub status: String,
    pub owner_id: String,
    pub owner_name: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_by_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResponse {
    pub deleted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    pub pdf_id: String,
    pub pdf_url: String,
    pub status: String,
}
