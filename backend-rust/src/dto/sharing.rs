use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct UserSearchResult {
    pub id: String,
    pub username: String,
    pub name: String,
    pub dni: Option<String>,
    pub cargo: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ShareSyncRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ShareSearchData {
    pub compartidos: Vec<UserSearchResult>,
    pub encontrados: Vec<UserSearchResult>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ShareSearchResponse {
    pub data: ShareSearchData,
}
