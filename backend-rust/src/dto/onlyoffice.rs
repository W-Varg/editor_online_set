use serde::Serialize;

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
    #[serde(rename = "plugins")]
    pub plugins: Option<OnlyOfficePlugins>,
    #[serde(rename = "user")]
    pub user: OnlyOfficeUser,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeCustomization {
    pub autosave: bool,
    pub forcesave: bool,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pluginsData")]
    pub plugins_data: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficePlugins {
    pub autostart: bool,
    pub plugins: Vec<OnlyOfficePluginItem>,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficePluginItem {
    pub id: String,
    pub src: String,
}

#[derive(Debug, Serialize)]
pub struct OnlyOfficeUser {
    pub id: String,
    pub name: String,
}
