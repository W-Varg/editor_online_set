use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeConfig {
    pub document: OnlyOfficeDocument,
    #[serde(rename = "documentType")]
    pub document_type: String,
    #[serde(rename = "editorConfig")]
    pub editor_config: OnlyOfficeEditorConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeDocument {
    #[serde(rename = "fileType")]
    pub file_type: String,
    pub key: String,
    pub title: String,
    pub url: String,
    pub permissions: OnlyOfficePermissions,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficePermissions {
    pub edit: bool,
    #[serde(rename = "comment")]
    pub comment: bool,
    pub download: bool,
    pub print: bool,
    pub review: bool,
}

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeCustomization {
    pub autosave: bool,
    pub forcesave: bool,
}

/// Configuración de plugins de ONLYOFFICE (formato moderno, DocumentServer v7+).
///
/// - `autostart`: lista de GUIDs de plugins que arrancan automáticamente al abrir
///   el editor, en orden.
/// - `plugins_data`: lista de URLs absolutas a los `config.json` de cada plugin.
///   El editor las descarga y registra los plugins en la pestaña "Plugins".
/// - `options`: datos personalizados que recibe cada plugin, indexados por GUID
///   (`"asc.{...}": { ... }`). El plugin los lee como `Asc.plugin.info.options`.
///
/// Más información:
/// https://api.onlyoffice.com/docs/docs-api/usage-api/config/editor/plugins/
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficePlugins {
    pub autostart: Vec<String>,
    #[serde(rename = "pluginsData")]
    pub plugins_data: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeUser {
    pub id: String,
    pub name: String,
}
