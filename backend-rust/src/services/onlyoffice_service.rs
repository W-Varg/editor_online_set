use crate::db::DbConn;
use crate::dto::OnlyOfficeConfig;
use crate::helpers::{config, plugins, url};
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn document_type_for(ext: &str) -> &str {
    match ext {
        "docx" | "doc" => "word",
        "xlsx" | "xls" => "cell",
        "pptx" | "ppt" => "slide",
        _ => "word",
    }
}

fn editor_menus(document_type: &str) -> Vec<String> {
    let menus: Vec<&str> = match document_type {
        "cell" => vec!["Home", "Insert", "Layout", "Data", "Collaboration", "Plugins", "Macros", "View"],
        "slide" => vec!["Home", "Insert", "Layout", "Collaboration", "Plugins", "Macros", "View"],
        _ => vec!["Home", "Insert", "Layout", "References", "Collaboration", "Plugins", "Macros", "View"],
    };
    menus.into_iter().map(|m| m.to_string()).collect()
}

pub fn get_config(
    db: &DbConn,
    db_path: &PathBuf,
    headers: &HeaderMap,
    doc_id: &str,
    user_id: &str,
    user_name: &str,
    api_token: &str,
) -> Option<OnlyOfficeConfig> {
    let doc = super::document_service::get_by_id(db, doc_id)?;
    if doc.status == "final" || doc.ext == "pdf" {
        return None;
    }
    let content = crate::repos::document_repo::read_file(db_path, doc_id)?;
    let backend_url = config::public_backend_url(8091);
    let browser_url = url::public_service_url(headers, 8091);

    let document_type = document_type_for(&doc.ext);

    // Plugins personalizados: se filtran por tipo de documento para que cada
    // plugin solo se ofrezca en los editores que lo soportan. La URL pública se
    // construye con el host de la petición (funciona en localhost e intranet).
    let mut active_plugins = plugins::matching_plugins(&document_type);
    let is_owner = crate::repos::document_repo::is_owner(db, doc_id, user_id);
    // Los plugins marcados como `requires_owner` solo se inyectan al propietario.
    active_plugins.retain(|plugin| !plugin.requires_owner || is_owner);
    for plugin in &active_plugins {
        tracing::debug!("Aplicando plugin \"{}\" ({}).", plugin.name, plugin.id);
    }

    // URLs de los `config.json` que descargará el editor.
    let plugins_data: Vec<String> = active_plugins
        .iter()
        .map(|plugin| format!("{}/plugins/{}/config.json", browser_url, plugin.dir))
        .collect();
    // GUIDs de los plugins que arrancan automáticamente.
    let autostart: Vec<String> = active_plugins
        .iter()
        .filter(|plugin| plugin.autostart)
        .map(|plugin| plugin.id.to_string())
        .collect();

    // Opciones personalizadas por plugin (`editorConfig.plugins.options`).
    let ctx = plugins::PluginContext {
        doc_id,
        token: api_token,
        backend_url: &browser_url,
    };
    let options: serde_json::Map<String, serde_json::Value> = active_plugins
        .iter()
        .filter_map(|plugin| {
            plugin
                .options
                .map(|builder| (plugin.id.to_string(), builder(&ctx)))
        })
        .collect();
    let options = if options.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(options))
    };

    let mut hasher = Sha256::new();
    hasher.update(doc_id.as_bytes());
    hasher.update(b":");
    hasher.update(&content);
    let key = format!("{}-{:x}", doc_id, hasher.finalize());

    // Menús de la cabecera del editor sin "File" para ocultar el menú "Archivo".
    let menus = editor_menus(&document_type);

    Some(crate::dto::OnlyOfficeConfig {
        document_type: document_type.to_string(),
        document: crate::dto::OnlyOfficeDocument {
            file_type: doc.ext.clone(),
            key,
            title: format!("{}.{}", doc.name, doc.ext),
            url: format!("{}/download/{}", backend_url, doc_id),
            permissions: crate::dto::OnlyOfficePermissions {
                edit: true,
                comment: false,
                download: true,
                print: true,
                review: false,
            },
        },
        editor_config: crate::dto::OnlyOfficeEditorConfig {
            callback_url: format!("{}/callback/onlyoffice/{}", backend_url, doc_id),
            lang: "es-ES".to_string(),
            mode: "edit".to_string(),
            customization: crate::dto::OnlyOfficeCustomization {
                autosave: true,
                forcesave: true,
                menu: Some(menus),
            },
            // Solo se incluye la sección `plugins` si hay plugins que aplicar al
            // tipo de documento actual; si la lista queda vacía se omite el campo.
            plugins: (!active_plugins.is_empty()).then(|| crate::dto::OnlyOfficePlugins {
                autostart,
                plugins_data,
                options,
            }),
            user: crate::dto::OnlyOfficeUser {
                id: user_id.to_string(),
                name: user_name.to_string(),
            },
        },
        token: None,
    })
}

/// Genera la configuración de ONLYOFFICE para *editar una plantilla*.
///
/// Las plantillas son globales y solo se editan con ONLYOFFICE. En el editor se
/// inyecta únicamente el plugin "Etiquetas" (`TEMPLATE_PLUGINS`) para insertar
/// `{{key}}` en el contenido; el callback y el origen del archivo apuntan a los
/// endpoints de plantillas (`/api/templates/{id}/content` y `/callback/template/{id}`).
pub fn get_template_config(
    db: &DbConn,
    db_path: &PathBuf,
    headers: &HeaderMap,
    template_id: &str,
    user_id: &str,
    user_name: &str,
    api_token: &str,
) -> Option<OnlyOfficeConfig> {
    let template = crate::repos::template_repo::get_by_id(db, template_id)?;
    let content = crate::repos::template_repo::read_file(db_path, template_id)?;
    let backend_url = config::public_backend_url(8091);
    let browser_url = url::public_service_url(headers, 8091);

    let document_type = document_type_for(&template.ext);

    // Plugins para plantillas: solo los incluidos en TEMPLATE_PLUGINS (Etiquetas).
    // El navegador descarga los `config.json`, así que se usa la URL pública.
    let active_plugins = plugins::template_plugins(&document_type);
    let plugins_data: Vec<String> = active_plugins
        .iter()
        .map(|plugin| format!("{}/plugins/{}/config.json", browser_url, plugin.dir))
        .collect();
    let autostart: Vec<String> = active_plugins
        .iter()
        .filter(|plugin| plugin.autostart)
        .map(|plugin| plugin.id.to_string())
        .collect();
    let ctx = plugins::PluginContext {
        doc_id: template_id,
        token: api_token,
        backend_url: &backend_url,
    };
    let options: serde_json::Map<String, serde_json::Value> = active_plugins
        .iter()
        .filter_map(|plugin| {
            plugin
                .options
                .map(|builder| (plugin.id.to_string(), builder(&ctx)))
        })
        .collect();
    let options = if options.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(options))
    };

    let mut hasher = Sha256::new();
    hasher.update(template_id.as_bytes());
    hasher.update(b":");
    hasher.update(&content);
    let key = format!("{}-{:x}", template_id, hasher.finalize());

    let menus = editor_menus(&document_type);

    Some(crate::dto::OnlyOfficeConfig {
        document_type: document_type.to_string(),
        document: crate::dto::OnlyOfficeDocument {
            file_type: template.ext.clone(),
            key,
            title: format!("{}.{}", template.name, template.ext),
            url: format!("{}/api/templates/{}/content", backend_url, template_id),
            permissions: crate::dto::OnlyOfficePermissions {
                edit: true,
                comment: false,
                download: true,
                print: true,
                review: false,
            },
        },
        editor_config: crate::dto::OnlyOfficeEditorConfig {
            callback_url: format!("{}/callback/template/{}", backend_url, template_id),
            lang: "es-ES".to_string(),
            mode: "edit".to_string(),
            customization: crate::dto::OnlyOfficeCustomization {
                autosave: true,
                forcesave: true,
                menu: Some(menus),
            },
            plugins: (!active_plugins.is_empty()).then(|| crate::dto::OnlyOfficePlugins {
                autostart,
                plugins_data,
                options,
            }),
            user: crate::dto::OnlyOfficeUser {
                id: user_id.to_string(),
                name: user_name.to_string(),
            },
        },
        token: None,
    })
}
