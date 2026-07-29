use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub id: String,
    pub document_id: String,
    pub user_id: String,
    pub user_name: String,
    pub shared_by: String,
    pub shared_by_name: String,
    pub permission: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserSearchResult {
    pub id: String,
    pub username: String,
    pub name: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
}
