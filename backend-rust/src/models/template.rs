use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub editor: String,
    pub size: u64,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}
