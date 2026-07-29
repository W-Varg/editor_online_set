use std::path::PathBuf;
use crate::db::DbConn;
use crate::dto::{DocumentResponse, CreateDocument};
use crate::models::Document;
use crate::repos::document_repo;

fn doc_ext_to_mime(ext: &str) -> &str {
    match ext {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentationml",
        _ => "application/octet-stream",
    }
}

pub fn create(db: &DbConn, db_path: &PathBuf, req: &CreateDocument, owner_id: &str) -> Result<Document, String> {
    let ext = req.ext.to_lowercase();
    let mime = doc_ext_to_mime(&ext).to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let content = match ext.as_str() {
        "docx" => crate::templates::generate_blank_docx(),
        "xlsx" => crate::templates::generate_blank_xlsx(),
        _ => return Err("Unsupported extension".to_string()),
    };

    let doc = Document {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        ext,
        mime,
        editor: req.editor.clone(),
        size: content.len() as u64,
        status: "draft".to_string(),
        owner_id: owner_id.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    document_repo::write_file(db_path, &doc, &content)?;
    document_repo::insert(db, &doc)?;
    Ok(doc)
}

pub fn get_mine(db: &DbConn, user_id: &str) -> Vec<DocumentResponse> {
    document_repo::list_mine(db, user_id)
}

pub fn get_shared(db: &DbConn, user_id: &str) -> Vec<DocumentResponse> {
    document_repo::list_shared(db, user_id)
}

pub fn get_by_id(db: &DbConn, doc_id: &str) -> Option<Document> {
    document_repo::get_by_id(db, doc_id)
}

pub fn delete(db: &DbConn, doc_id: &str, user_id: &str, db_path: &PathBuf) -> Result<bool, String> {
    if document_repo::is_owner(db, doc_id, user_id) {
        if document_repo::shared_count(db, doc_id) > 0 {
            document_repo::remove_share_entry(db, doc_id, user_id)?;
            Ok(false)
        } else {
            document_repo::physical_delete(db, doc_id, db_path);
            Ok(true)
        }
    } else {
        document_repo::remove_share_entry(db, doc_id, user_id)?;
        Ok(false)
    }
}
