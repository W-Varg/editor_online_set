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

/// Descripción declarativa de un plugin personalizado servido por el backend.
///
/// - `id`: GUID único del plugin (debe coincidir con `guid` en su `config.json`).
///   Se usa en `editorConfig.plugins.autostart` para que el plugin arranque solo.
/// - `name`: nombre para la documentación y futuros usos.
/// - `dir`: subcarpeta dentro de `backend-rust/public/plugins/` que contiene el
///   plugin. Con ella se construye la URL pública del `config.json`.
/// - `editors`: tipos de documento en los que el plugin debe estar disponible.
///   Valores válidos: `"word"`, `"cell"`, `"slide"`, `"pdf"`.
/// - `autostart`: si es `true`, el GUID entra en `plugins.autostart` y el plugin
///   se ejecuta automáticamente al abrir el editor. Si es `false` queda
///   disponible en la pestaña "Plugins" para abrirlo con un clic.
#[derive(Debug, Clone, Copy)]
pub struct CustomPlugin {
    pub id: &'static str,
    pub name: &'static str,
    pub dir: &'static str,
    pub editors: &'static [&'static str],
    pub autostart: bool,
}

/// Catálogo de plugins personalizados.
///
/// Se mantiene en una sola constante para que el alta de plugins sea declarativa
/// y no requiera tocar la lógica de generación de configuración.
pub const CUSTOM_PLUGINS: &[CustomPlugin] = &[
    // Plugin de ejemplo "Saludar": sidebar con un cuadro de texto que abre una
    // ventana modal con el saludo. Vive en public/plugins/saludar/.
    CustomPlugin {
        id: "asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}",
        name: "Saludar",
        dir: "saludar",
        editors: &["word", "cell", "slide"],
        autostart: true,
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
