//! Registro central de plugins personalizados para ONLYOFFICE.
//!
//! DocumentServer carga los plugins de forma *externa*: el backend sirve los
//! archivos del plugin como estáticos (ruta `/plugins/<directorio>/`) y luego
//! inyecta la URL del `config.json` de cada plugin en la configuración que le
//! envía al editor (`editorConfig.plugins.pluginsData`).
//!
//! Para agregar un plugin nuevo basta con:
//!
//! 1. Crear la carpeta `backend-rust/public/plugins/<directorio>/` con los
//!    archivos del plugin (`config.json`, `index.html`, `plugin.js`, ...).
//! 2. Registrar una entrada [`CustomPlugin`] en [`CUSTOM_PLUGINS`].
//!
//! El servicio `onlyoffice_service::get_config` filtra los plugins por el tipo
//! de documento abierto (`word`, `cell`, `slide`) usando [`matching_plugins`],
//! de modo que cada plugin solo se ofrece en los editores que lo soportan.

/// Contexto que reciben los generadores de opciones ([`CustomPlugin::options`]).
///
/// Contiene los datos que un plugin puede necesitar de forma dinámica según la
/// sesión actual, por ejemplo para llamar a la API del backend autenticado.
#[derive(Debug, Clone, Copy)]
pub struct PluginContext<'a> {
    /// Identificador del documento que se está editando.
    pub doc_id: &'a str,
    /// JWT del usuario autenticado (para autorizar llamadas al backend).
    pub token: &'a str,
    /// URL pública del backend accesible desde el navegador del usuario.
    pub backend_url: &'a str,
}

/// Constructor de las opciones que el editor le pasará al plugin a través de
/// `editorConfig.plugins.options[guid]` (leídas por el plugin como
/// `window.Asc.plugin.info.options`).
pub type PluginOptionsBuilder = fn(&PluginContext) -> serde_json::Value;

/// Descripción declarativa de un plugin personalizado servido por el backend.
///
/// - `id`: GUID único del plugin (debe coincidir con `guid` en su `config.json`).
///   Se usa en `editorConfig.plugins.autostart` para que el plugin arranque solo.
/// - `name`: nombre para la documentación y logs.
/// - `dir`: subcarpeta dentro de `backend-rust/public/plugins/` que contiene el
///   plugin. Con ella se construye la URL pública del `config.json`.
/// - `editors`: tipos de documento en los que el plugin debe estar disponible.
///   Valores válidos: `"word"`, `"cell"`, `"slide"`, `"pdf"`.
/// - `autostart`: si es `true`, el GUID entra en `plugins.autostart` y el plugin
///   se ejecuta automáticamente al abrir el editor. Si es `false` queda
///   disponible en la pestaña "Plugins" para abrirlo con un clic (el sidebar
///   no se abre solo).
/// - `requires_owner`: si es `true`, el plugin solo se inyecta cuando el usuario
///   autenticado es el propietario del documento (p. ej. administrar accesos).
/// - `options`: generador opcional de `editorConfig.plugins.options[guid]`.
#[derive(Debug, Clone, Copy)]
pub struct CustomPlugin {
    pub id: &'static str,
    pub name: &'static str,
    pub dir: &'static str,
    pub editors: &'static [&'static str],
    pub autostart: bool,
    pub requires_owner: bool,
    pub options: Option<PluginOptionsBuilder>,
}

/// Genera las opciones que recibe el plugin "Compartir": el documento, el JWT
/// del usuario y la URL pública del backend para poder llamar a la API.
fn build_compartir_options(ctx: &PluginContext) -> serde_json::Value {
    serde_json::json!({
        "docId": ctx.doc_id,
        "token": ctx.token,
        "backendUrl": ctx.backend_url,
    })
}

/// Genera las opciones para el plugin "Etiquetas": el JWT del usuario y la URL
/// pública del backend para poder consultar `GET /api/tags`.
fn build_etiquetas_options(ctx: &PluginContext) -> serde_json::Value {
    serde_json::json!({
        "token": ctx.token,
        "backendUrl": ctx.backend_url,
    })
}

/// Catálogo de plugins personalizados.
///
/// Se mantiene en una sola constante para que el alta de plugins sea declarativa
/// y no requiera tocar la lógica de generación de configuración.
pub const CUSTOM_PLUGINS: &[CustomPlugin] = &[
    // Plugin de ejemplo "Saludar": sidebar con un cuadro de texto que abre una
    // ventana modal con el saludo. Vive en public/plugins/saludar/.
    //
    // `autostart: false`: el plugin queda disponible en la pestaña "Plugins"
    // (se "carga por defecto" al abrir el editor) pero el sidebar NO se abre
    // solo; se muestra al hacer clic en el plugin.
    CustomPlugin {
        id: "asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}",
        name: "Saludar",
        dir: "saludar",
        editors: &["word", "cell", "slide"],
        autostart: false,
        requires_owner: false,
        options: None,
    },
    // Plugin "Compartir": sidebar para buscar usuarios por DNI/nombre y otorgar
    // o revocar acceso al documento. Vive en public/plugins/compartir/.
    //
    // `requires_owner: true`: solo aparece para el propietario del documento,
    // que es quien puede administrar los permisos según el backend.
    CustomPlugin {
        id: "asc.{8f2a1c40-7b3d-4e21-9a6f-000000000001}",
        name: "Compartir",
        dir: "compartir",
        editors: &["word", "cell", "slide"],
        autostart: false,
        requires_owner: true,
        options: Some(build_compartir_options),
    },
    // Plugin "Etiquetas": inserta etiquetas dinámicas ({{key}}) en el contenido.
    // Vive en public/plugins/etiquetas/. El backend las resuelve al previsualizar
    // o convertir a PDF, por lo que el plugin no requiere ser propietario.
    CustomPlugin {
        id: "asc.{8f2a1c40-7b3d-4e21-9a6f-000000000003}",
        name: "Etiquetas",
        dir: "etiquetas",
        editors: &["word", "cell"],
        autostart: false,
        requires_owner: false,
        options: Some(build_etiquetas_options),
    },
];

/// Devuelve los plugins que soportan el tipo de documento indicado.
///
/// `document_type` es uno de los valores que devuelve el editor de ONLYOFFICE
/// ("word", "cell", "slide", "pdf") según la extensión del archivo.
pub fn matching_plugins(document_type: &str) -> Vec<&'static CustomPlugin> {
    CUSTOM_PLUGINS
        .iter()
        .filter(|plugin| plugin.editors.contains(&document_type))
        .collect()
}
