use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct TagDefinition {
    pub key: String,
    pub label: String,
    pub description: String,
}
