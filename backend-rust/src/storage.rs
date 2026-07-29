use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::models::{Document, User};
use crate::templates;

#[derive(Clone)]
pub struct Storage {
    db: std::sync::Arc<Mutex<Connection>>,
    data_path: PathBuf,
}

impl Storage {
    pub fn new(data_dir: &str) -> Self {
        let data_path = PathBuf::from(data_dir);
        std::fs::create_dir_all(&data_path).unwrap_or_default();

        let db_path = data_path.join("editor.db");
        let conn = Connection::open(&db_path).expect("Failed to open SQLite database");

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ext TEXT NOT NULL,
                mime TEXT NOT NULL,
                editor TEXT NOT NULL DEFAULT 'onlyoffice',
                size INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'draft',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );"
        ).expect("Failed to create tables");

        Self {
            db: std::sync::Arc::new(Mutex::new(conn)),
            data_path,
        }
    }

    pub fn init(&self) {
        let db = self.db.lock().unwrap();
        let user_count: i64 = db.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap_or(0);
        drop(db);

        if user_count == 0 {
            self.seed_users();
        }
    }

    fn seed(&self, name: &str, ext: &str, editor: &str) {
        let content = match ext {
            "docx" => templates::generate_sample_docx(),
            "xlsx" => templates::generate_sample_xlsx(),
            _ => return,
        };
        let id = uuid::Uuid::new_v4().to_string();
        let mime = match ext {
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            _ => "application/octet-stream",
        };
        let now = chrono::Utc::now().to_rfc3339();

        std::fs::write(self.doc_path(&id), &content).unwrap_or_default();

        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO documents (id, name, ext, mime, editor, size, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![id, name, ext, mime, editor, content.len() as u64, "draft", now.clone(), now],
        ).unwrap_or_default();
    }

    fn seed_users(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();
        for i in 1..=5 {
            let id = uuid::Uuid::new_v4().to_string();
            db.execute(
                "INSERT INTO users (id, username, password, name, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, format!("user{}", i), "Admin123@", format!("User {}", i), now.clone()],
            ).unwrap_or_default();
        }
    }

    fn doc_path(&self, id: &str) -> PathBuf {
        self.data_path.join(format!("{}.bin", id))
    }

    fn pdf_path(&self, id: &str) -> PathBuf {
        self.data_path.join(format!("{}.pdf", id))
    }

    // ---- Users ----

    pub fn authenticate(&self, username: &str, password: &str) -> Option<User> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT id, username, name FROM users WHERE username = ?1 AND password = ?2",
            rusqlite::params![username, password],
            |row| Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                name: row.get(2)?,
            }),
        ).ok()
    }

    // ---- Documents ----

    pub fn create_document(&self, name: &str, ext: &str, editor: &str) -> Result<Document, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mime = match ext {
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            _ => "application/octet-stream",
        };
        let content = match ext {
            "docx" => templates::generate_blank_docx(),
            "xlsx" => templates::generate_blank_xlsx(),
            _ => return Err("Unsupported extension".to_string()),
        };
        let now = chrono::Utc::now().to_rfc3339();

        let doc = Document {
            id: id.clone(),
            name: name.to_string(),
            ext: ext.to_string(),
            mime: mime.to_string(),
            editor: editor.to_string(),
            size: content.len() as u64,
            status: "draft".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        std::fs::write(self.doc_path(&id), &content).map_err(|e| e.to_string())?;

        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO documents (id, name, ext, mime, editor, size, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                doc.id, doc.name, doc.ext, doc.mime, doc.editor,
                doc.size, doc.status, doc.created_at, doc.updated_at
            ],
        ).map_err(|e| e.to_string())?;

        Ok(doc)
    }

    pub fn list_documents(&self) -> Vec<Document> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT id, name, ext, mime, editor, size, status, created_at, updated_at
             FROM documents ORDER BY created_at DESC"
        ).unwrap();
        stmt.query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                ext: row.get(2)?,
                mime: row.get(3)?,
                editor: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn get_document(&self, id: &str) -> Option<Document> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT id, name, ext, mime, editor, size, status, created_at, updated_at
             FROM documents WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Document {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ext: row.get(2)?,
                    mime: row.get(3)?,
                    editor: row.get(4)?,
                    size: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        ).ok()
    }

    pub fn get_document_content(&self, id: &str) -> Option<Vec<u8>> {
        let path = self.doc_path(id);
        if !path.exists() { return None; }
        std::fs::read(path).ok()
    }

    pub fn update_document_content(&self, id: &str, content: &[u8]) -> Result<(), String> {
        let path = self.doc_path(id);
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();
        db.execute(
            "UPDATE documents SET size = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![content.len() as u64, now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_document(&self, id: &str) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM documents WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
        drop(db);

        let _ = std::fs::remove_file(self.doc_path(id));
        let _ = std::fs::remove_file(self.pdf_path(id));

        Ok(())
    }

    pub fn pdf_exists(&self, id: &str) -> bool {
        self.pdf_path(id).exists()
    }

    pub fn get_pdf_content(&self, id: &str) -> Option<Vec<u8>> {
        let path = self.pdf_path(id);
        if !path.exists() { return None; }
        std::fs::read(path).ok()
    }

    pub fn save_pdf(&self, id: &str, content: &[u8]) -> Result<(), String> {
        std::fs::write(self.pdf_path(id), content).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();
        db.execute(
            "UPDATE documents SET status = 'final', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
