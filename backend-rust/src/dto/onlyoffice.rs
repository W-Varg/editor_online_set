use serde::Serialize;
use utoipa::ToSchema;

/// Configuración completa para inicializar el editor de ONLYOFFICE en el navegador.
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeConfig {
    /// Información del documento a editar.
    pub document: OnlyOfficeDocument,
    /// Tipo de documento: `word`, `cell`, `slide` o `pdf`.
    #[serde(rename = "documentType")]
    #[schema(example = "word")]
    pub document_type: String,
    /// Configuración del editor (usuario, permisos, plugins, etc.).
    #[serde(rename = "editorConfig")]
    pub editor_config: OnlyOfficeEditorConfig,
    /// JWT firmado por el backend que protege la configuración frente a ONLYOFFICE.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...", value_type = String)]
    pub token: Option<String>,
}

/// Información del documento que verá ONLYOFFICE.
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeDocument {
    /// Extensión del archivo (docx, xlsx, odt...).
    #[serde(rename = "fileType")]
    #[schema(example = "docx")]
    pub file_type: String,
    /// Clave única del documento (cambia al guardar para invalidar caché).
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789-1754160000")]
    pub key: String,
    /// Título mostrado en la cabecera del editor.
    #[schema(example = "Contrato de servicios.docx")]
    pub title: String,
    /// URL pública desde la que ONLYOFFICE descarga el contenido inicial.
    #[schema(example = "http://localhost:8091/api/documents/0a1b2c3d.../content")]
    pub url: String,
    /// Permisos del usuario sobre el documento.
    pub permissions: OnlyOfficePermissions,
}

/// Permisos del usuario sobre el documento en ONLYOFFICE.
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficePermissions {
    /// Permite editar el documento.
    #[schema(example = true)]
    pub edit: bool,
    /// Permite comentar el documento.
    #[serde(rename = "comment")]
    #[schema(example = true)]
    pub comment: bool,
    /// Permite descargar el documento.
    #[schema(example = true)]
    pub download: bool,
    /// Permite imprimir el documento.
    #[schema(example = true)]
    pub print: bool,
    /// Permite aceptar/rechazar cambios marcados (revisión).
    #[schema(example = true)]
    pub review: bool,
}

/// Configuración del editor (modo, idioma, usuario y callback de guardado).
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeEditorConfig {
    /// URL del callback al que ONLYOFFICE notifica el guardado.
    #[serde(rename = "callbackUrl")]
    #[schema(example = "http://localhost:8091/callback/onlyoffice/0a1b2c3d...")]
    pub callback_url: String,
    /// Código de idioma de la interfaz del editor.
    #[schema(example = "es")]
    pub lang: String,
    /// Modo del editor: `view`, `edit` o `comment`.
    #[schema(example = "edit")]
    pub mode: String,
    /// Personalizaciones de la interfaz.
    #[serde(rename = "customization")]
    pub customization: OnlyOfficeCustomization,
    /// Configuración de plugins del editor (si hay).
    #[serde(rename = "plugins")]
    pub plugins: Option<OnlyOfficePlugins>,
    /// Usuario que abre el documento.
    #[serde(rename = "user")]
    pub user: OnlyOfficeUser,
}

/// Personalizaciones visuales del editor de ONLYOFFICE.
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeCustomization {
    /// Activa el guardado automático del documento.
    #[schema(example = true)]
    pub autosave: bool,
    /// Muestra el botón "Guardar" (forcesave) en la interfaz.
    #[schema(example = true)]
    pub forcesave: bool,
    /// Menús del editor que se mostrarán en la cabecera. Si se omite, se muestran
    /// todos. Enviar la lista SIN "File" oculta el menú "Archivo".
    /// Disponible desde DocumentServer v7.3.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = json!(["File", "Home"]), value_type = Vec<String>)]
    pub menu: Option<Vec<String>>,
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
    /// GUIDs de los plugins que se inician automáticamente.
    #[schema(example = json!(["asc.{b5c84b3e-...-0bcd9ef88c2f}"]), value_type = Vec<String>)]
    pub autostart: Vec<String>,
    /// URLs absolutas a los `config.json` de cada plugin a registrar.
    #[serde(rename = "pluginsData")]
    #[schema(example = json!(["http://localhost:8091/plugins/etiquetas/config.json"]), value_type = Vec<String>)]
    pub plugins_data: Vec<String>,
    /// Datos personalizados por plugin, indexados por GUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

/// Usuario que abre el documento en ONLYOFFICE.
#[derive(Debug, Serialize, ToSchema)]
pub struct OnlyOfficeUser {
    /// Identificador único del usuario.
    #[schema(example = "1")]
    pub id: String,
    /// Nombre visible del usuario.
    #[schema(example = "Juan Pérez")]
    pub name: String,
}
