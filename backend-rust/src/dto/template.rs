use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTemplate {
    pub name: String,
    pub ext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_document_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameTemplate {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub editor: String,
    pub size: u64,
    pub owner_id: String,
    pub owner_name: String,
    pub created_at: String,
    pub updated_at: String,
}
