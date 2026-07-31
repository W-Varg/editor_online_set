use crate::db::DbConn;
use crate::dto::OnlyOfficeConfig;
use crate::helpers::{config, url};
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

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

    let document_type = match doc.ext.as_str() {
        "docx" | "doc" => "word",
        "xlsx" | "xls" => "cell",
        "pptx" | "ppt" => "slide",
        _ => "word",
    };

    let mut hasher = Sha256::new();
    hasher.update(doc_id.as_bytes());
    hasher.update(b":");
    hasher.update(&content);
    let key = format!("{}-{:x}", doc_id, hasher.finalize());

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
                plugins_data: Some(vec![
                    vec![
                        doc_id.to_string(),
                        api_token.to_string(),
                        browser_url.clone(),
                    ],
                ]),
            },
            plugins: Some(crate::dto::OnlyOfficePlugins {
                autostart: false,
                plugins: vec![
                    crate::dto::OnlyOfficePluginItem {
                        id: "asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}".to_string(),
                        src: "http://localhost:8092/sdkjs-plugins/saludar/config.json".to_string(),
                        name: Some("Saludar".to_string()),
                    },
                ],
            }),
            user: crate::dto::OnlyOfficeUser {
                id: user_id.to_string(),
                name: user_name.to_string(),
            },
        },
        token: None,
    })
}
