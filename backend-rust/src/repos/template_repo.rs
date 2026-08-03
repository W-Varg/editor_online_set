use std::path::PathBuf;
use rusqlite::params;
use crate::db::DbConn;
use crate::dto::TemplateResponse;
use crate::models::Template;

fn template_path(data_path: &PathBuf, id: &str) -> PathBuf {
    data_path.join("templates").join(format!("{}.bin", id))
}

pub fn insert(db: &DbConn, template: &Template) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO templates (id, name, ext, mime, editor, size, owner_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![template.id, template.name, template.ext, template.mime,
                template.editor, template.size, template.owner_id,
                template.created_at, template.updated_at],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_by_id(db: &DbConn, id: &str) -> Option<Template> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, name, ext, mime, editor, size, owner_id, created_at, updated_at
         FROM templates WHERE id = ?1",
        params![id],
        |row| Ok(Template {
            id: row.get(0)?,
            name: row.get(1)?,
            ext: row.get(2)?,
            mime: row.get(3)?,
            editor: row.get(4)?,
            size: row.get(5)?,
            owner_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }),
    ).ok()
}

pub fn list_all(db: &DbConn) -> Vec<TemplateResponse> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.ext, t.mime, t.editor, t.size, t.owner_id,
                COALESCE(u.name, ''), t.created_at, t.updated_at
         FROM templates t
         LEFT JOIN users u ON u.id = t.owner_id
         ORDER BY t.updated_at DESC"
    ).unwrap();
    stmt.query_map([], |row| {
        Ok(TemplateResponse {
            id: row.get(0)?, name: row.get(1)?, ext: row.get(2)?,
            mime: row.get(3)?, editor: row.get(4)?, size: row.get(5)?,
            owner_id: row.get(6)?, owner_name: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn rename(db: &DbConn, id: &str, name: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.lock().unwrap();
    let affected = conn.execute(
        "UPDATE templates SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, now, id],
    ).map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("Plantilla no encontrada".to_string());
    }
    Ok(())
}

pub fn update_size(db: &DbConn, id: &str, size: u64) {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE templates SET size = ?1, updated_at = ?2 WHERE id = ?3",
        params![size, now, id],
    ).unwrap_or_default();
}

pub fn physical_delete(db: &DbConn, id: &str, data_path: &PathBuf) {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM templates WHERE id = ?1", params![id]).unwrap_or_default();
    let _ = std::fs::remove_file(template_path(data_path, id));
}

pub fn write_file(data_path: &PathBuf, template: &Template, content: &[u8]) -> Result<(), String> {
    let path = template_path(data_path, &template.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_default();
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

pub fn read_file(data_path: &PathBuf, id: &str) -> Option<Vec<u8>> {
    let path = template_path(data_path, id);
    if path.exists() { std::fs::read(path).ok() } else { None }
}
