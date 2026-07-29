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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocument {
    pub name: String,
    pub ext: String,
    pub editor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct CollaboraSession {
    pub iframe_url: String,
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeConfig {
    pub document: OnlyOfficeDocument,
    #[serde(rename = "documentType")]
    pub document_type: String,
    #[serde(rename = "editorConfig")]
    pub editor_config: OnlyOfficeEditorConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeDocument {
    #[serde(rename = "fileType")]
    pub file_type: String,
    pub key: String,
    pub title: String,
    pub url: String,
    pub permissions: OnlyOfficePermissions,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficePermissions {
    pub edit: bool,
    #[serde(rename = "comment")]
    pub comment: bool,
    pub download: bool,
    pub print: bool,
    pub review: bool,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeEditorConfig {
    #[serde(rename = "callbackUrl")]
    pub callback_url: String,
    pub lang: String,
    pub mode: String,
    #[serde(rename = "customization")]
    pub customization: OnlyOfficeCustomization,
    #[serde(rename = "user")]
    pub user: OnlyOfficeUser,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeCustomization {
    pub autosave: bool,
    pub forcesave: bool,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeUser {
    pub id: String,
    pub name: String,
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

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub pdf_id: String,
    pub pdf_url: String,
    pub status: String,
}
