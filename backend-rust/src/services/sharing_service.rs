use crate::db::DbConn;
use crate::dto::ShareResponse;
use crate::repos::document_repo;

pub fn share(db: &DbConn, doc_id: &str, user_id: &str, shared_by: &str) -> Result<ShareResponse, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();

    // Already shared?
    if document_repo::is_shared_with(db, doc_id, user_id) {
        return Err("El documento ya está compartido con este usuario".to_string());
    }
    // Can't share with self
    if document_repo::is_owner(db, doc_id, user_id) {
        return Err("No puedes compartir un documento contigo mismo".to_string());
    }
    // Doc must exist
    if document_repo::get_by_id(db, doc_id).is_none() {
        return Err("Documento no encontrado".to_string());
    }

    document_repo::insert_share(db, &id, doc_id, user_id, shared_by, &now)?;

    let user_name = document_repo::get_user_name(db, user_id);
    let shared_by_name = document_repo::get_user_name(db, shared_by);

    Ok(ShareResponse {
        id,
        document_id: doc_id.to_string(),
        user_id: user_id.to_string(),
        user_name,
        shared_by: shared_by.to_string(),
        shared_by_name,
        permission: "edit".to_string(),
        created_at: now,
    })
}

pub fn list(db: &DbConn, doc_id: &str) -> Vec<ShareResponse> {
    document_repo::list_shares(db, doc_id)
}

pub fn remove(db: &DbConn, doc_id: &str, user_id: &str) -> Result<(), String> {
    document_repo::remove_share_entry(db, doc_id, user_id)
}
