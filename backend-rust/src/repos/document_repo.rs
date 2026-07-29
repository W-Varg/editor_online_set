use std::path::PathBuf;
use rusqlite::params;
use crate::db::DbConn;
use crate::models::Document;
use crate::dto::{DocumentResponse, ShareResponse};

fn doc_path(data_path: &PathBuf, id: &str) -> PathBuf {
    data_path.join(format!("{}.bin", id))
}

fn file_path(data_path: &PathBuf, id: &str) -> PathBuf {
    if let Ok(entries) = std::fs::read_dir(data_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(id) && name != format!("{}.pdf", id) && name != format!("{}.bin", id) {
                return entry.path();
            }
        }
    }
    doc_path(data_path, id)
}

pub fn insert(db: &DbConn, doc: &Document) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO documents (id, name, ext, mime, editor, size, status, owner_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![doc.id, doc.name, doc.ext, doc.mime, doc.editor,
                doc.size, doc.status, doc.owner_id, doc.created_at, doc.updated_at],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_by_id(db: &DbConn, id: &str) -> Option<Document> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, name, ext, mime, editor, size, status, owner_id, created_at, updated_at
         FROM documents WHERE id = ?1",
        params![id],
        |row| Ok(Document {
            id: row.get(0)?,
            name: row.get(1)?,
            ext: row.get(2)?,
            mime: row.get(3)?,
            editor: row.get(4)?,
            size: row.get(5)?,
            status: row.get(6)?,
            owner_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }),
    ).ok()
}

pub fn list_mine(db: &DbConn, user_id: &str) -> Vec<DocumentResponse> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.ext, d.mime, d.editor, d.size, d.status, d.owner_id,
                COALESCE(u.name, ''), d.created_at, d.updated_at
         FROM documents d
         LEFT JOIN users u ON u.id = d.owner_id
         WHERE d.owner_id = ?1
         ORDER BY d.updated_at DESC"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(DocumentResponse {
            id: row.get(0)?, name: row.get(1)?, ext: row.get(2)?,
            mime: row.get(3)?, editor: row.get(4)?, size: row.get(5)?,
            status: row.get(6)?, owner_id: row.get(7)?, owner_name: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
            shared: None, shared_by: None, shared_by_name: None,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn list_shared(db: &DbConn, user_id: &str) -> Vec<DocumentResponse> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.ext, d.mime, d.editor, d.size, d.status, d.owner_id,
                COALESCE(owner_u.name, ''), d.created_at, d.updated_at,
                s.shared_by, COALESCE(sharer_u.name, '')
         FROM document_shares s
         JOIN documents d ON d.id = s.document_id
         LEFT JOIN users owner_u ON owner_u.id = d.owner_id
         LEFT JOIN users sharer_u ON sharer_u.id = s.shared_by
         WHERE s.user_id = ?1
         ORDER BY s.created_at DESC"
    ).unwrap();
    stmt.query_map(params![user_id], |row| {
        Ok(DocumentResponse {
            id: row.get(0)?, name: row.get(1)?, ext: row.get(2)?,
            mime: row.get(3)?, editor: row.get(4)?, size: row.get(5)?,
            status: row.get(6)?, owner_id: row.get(7)?, owner_name: row.get(8)?,
            created_at: row.get(9)?, updated_at: row.get(10)?,
            shared: Some(true), shared_by: Some(row.get(11)?), shared_by_name: Some(row.get(12)?),
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn update_content(db: &DbConn, id: &str, size: u64) {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE documents SET size = ?1, updated_at = ?2 WHERE id = ?3",
        params![size, now, id],
    ).unwrap_or_default();
}

pub fn physical_delete(db: &DbConn, id: &str, data_path: &PathBuf) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM document_shares WHERE document_id = ?1", params![id]).unwrap_or_default();
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id]).unwrap_or_default();
    let _ = std::fs::remove_file(doc_path(data_path, id));
    let _ = std::fs::remove_file(data_path.join(format!("{}.pdf", id)));
}

pub fn remove_share_entry(db: &DbConn, doc_id: &str, user_id: &str) -> Result<(), String> {
    let conn = db.lock().unwrap();
    let affected = conn.execute(
        "DELETE FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
        params![doc_id, user_id],
    ).map_err(|e| format!("Error: {}", e))?;
    if affected == 0 { Err("No encontrado".to_string()) } else { Ok(()) }
}

pub fn is_owner(db: &DbConn, doc_id: &str, user_id: &str) -> bool {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE id = ?1 AND owner_id = ?2",
        params![doc_id, user_id],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

pub fn is_shared_with(db: &DbConn, doc_id: &str, user_id: &str) -> bool {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
        params![doc_id, user_id],
        |r| r.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

pub fn shared_count(db: &DbConn, doc_id: &str) -> i64 {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM document_shares WHERE document_id = ?1",
        params![doc_id],
        |r| r.get(0),
    ).unwrap_or(0)
}

pub fn insert_share(db: &DbConn, id: &str, doc_id: &str, user_id: &str, shared_by: &str, now: &str) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO document_shares (id, document_id, user_id, shared_by, permission, created_at)
         VALUES (?1, ?2, ?3, ?4, 'edit', ?5)",
        params![id, doc_id, user_id, shared_by, now],
    ).map_err(|e| format!("Error al compartir: {}", e))?;
    Ok(())
}

pub fn list_shares(db: &DbConn, doc_id: &str) -> Vec<ShareResponse> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT s.id, s.document_id, s.user_id, COALESCE(u.name, ''), s.shared_by,
                COALESCE(su.name, ''), s.permission, s.created_at
         FROM document_shares s
         LEFT JOIN users u ON u.id = s.user_id
         LEFT JOIN users su ON su.id = s.shared_by
         WHERE s.document_id = ?1
         ORDER BY s.created_at DESC"
    ).unwrap();
    stmt.query_map(params![doc_id], |row| {
        Ok(ShareResponse {
            id: row.get(0)?, document_id: row.get(1)?, user_id: row.get(2)?,
            user_name: row.get(3)?, shared_by: row.get(4)?, shared_by_name: row.get(5)?,
            permission: row.get(6)?, created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn get_user_name(db: &DbConn, id: &str) -> String {
    let conn = db.lock().unwrap();
    conn.query_row("SELECT name FROM users WHERE id = ?1", params![id], |r| r.get(0)).unwrap_or_default()
}

pub fn write_file(data_path: &PathBuf, doc: &Document, content: &[u8]) -> Result<(), String> {
    let path = doc_path(data_path, &doc.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_default();
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn read_file(data_path: &PathBuf, id: &str) -> Option<Vec<u8>> {
    let path = file_path(data_path, id);
    if path.exists() { std::fs::read(path).ok() } else { None }
}

pub fn pdf_path(data_path: &PathBuf, id: &str) -> PathBuf {
    data_path.join(format!("{}.pdf", id))
}
