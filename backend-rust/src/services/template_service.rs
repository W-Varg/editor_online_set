use std::path::PathBuf;
use crate::db::DbConn;
use crate::dto::{CreateTemplate, TemplateResponse};
use crate::models::Template;
use crate::repos::{document_repo, template_repo};

fn ext_to_mime(ext: &str) -> &str {
    match ext {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

pub fn create(
    db: &DbConn,
    db_path: &PathBuf,
    req: &CreateTemplate,
    owner_id: &str,
) -> Result<Template, String> {
    let ext = req.ext.to_lowercase();
    let mime = ext_to_mime(&ext).to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Editor de plantilla: onlyoffice por defecto, pero puede crearse con
    // Collabora para editar plantillas en ese editor (WOPI de plantillas).
    let editor = req.editor.as_deref().unwrap_or("onlyoffice").to_ascii_lowercase();
    if !matches!(editor.as_str(), "onlyoffice" | "collabora") {
        return Err("Editor no soportado".to_string());
    }

    // Si se indica un documento origen, la plantilla copia su contenido
    // (con etiquetas, encabezados/pies y todo lo editado). En caso contrario
    // se parte de un archivo en blanco del tipo indicado.
    let content = if let Some(source_id) = &req.source_document_id {
        document_repo::read_file(db_path, source_id)
            .ok_or_else(|| "Documento de origen no encontrado".to_string())?
    } else {
        match ext.as_str() {
            "docx" => crate::templates::generate_blank_docx(),
            "xlsx" => crate::templates::generate_blank_xlsx(),
            _ => return Err("Unsupported extension".to_string()),
        }
    };

    let template = Template {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        ext,
        mime,
        editor,
        size: content.len() as u64,
        owner_id: owner_id.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    template_repo::write_file(db_path, &template, &content)?;
    template_repo::insert(db, &template)?;
    Ok(template)
}

pub fn list(db: &DbConn) -> Vec<TemplateResponse> {
    template_repo::list_all(db)
}

pub fn get_by_id(db: &DbConn, id: &str) -> Option<Template> {
    template_repo::get_by_id(db, id)
}

pub fn rename(db: &DbConn, id: &str, name: &str) -> Result<Template, String> {
    template_repo::rename(db, id, name)?;
    template_repo::get_by_id(db, id).ok_or_else(|| "Plantilla no encontrada".to_string())
}

pub fn delete(db: &DbConn, id: &str, db_path: &PathBuf) -> Result<(), String> {
    if template_repo::get_by_id(db, id).is_none() {
        return Err("Plantilla no encontrada".to_string());
    }
    template_repo::physical_delete(db, id, db_path);
    Ok(())
}
