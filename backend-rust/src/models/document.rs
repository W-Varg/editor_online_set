use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub editor: String,
    pub size: u64,
    pub status: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}
