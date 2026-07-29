use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TokenQuery {
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CollaboraSession {
    pub iframe_url: String,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub file_id: String,
    pub name: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct CheckFileInfo {
    #[serde(rename = "BaseFileName")]
    pub base_file_name: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "UserId")]
    pub user_id: String,
    #[serde(rename = "UserFriendlyName")]
    pub user_friendly_name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "LastModifiedTime")]
    pub last_modified_time: String,
    #[serde(rename = "UserCanWrite")]
    pub user_can_write: bool,
    #[serde(rename = "UserCanNotWriteRelative")]
    pub user_can_not_write_relative: bool,
    #[serde(rename = "BreadcrumbDocName")]
    pub breadcrumb_doc_name: String,
    #[serde(rename = "SupportsLocks")]
    pub supports_locks: bool,
    #[serde(rename = "SupportsGetLock")]
    pub supports_get_lock: bool,
    #[serde(rename = "SupportsUpdate")]
    pub supports_update: bool,
    #[serde(rename = "SupportsExtendedLockLength")]
    pub supports_extended_lock_length: bool,
}
