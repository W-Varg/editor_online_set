use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, params};

use crate::models::*;
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

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap_or_default();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ext TEXT NOT NULL,
                mime TEXT NOT NULL,
                editor TEXT NOT NULL DEFAULT 'onlyoffice',
                size INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'draft',
                owner_id TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                name TEXT NOT NULL,
                dni TEXT,
                cargo TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS document_shares (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                shared_by TEXT NOT NULL,
                permission TEXT NOT NULL DEFAULT 'edit',
                created_at TEXT NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents(id),
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (shared_by) REFERENCES users(id),
                UNIQUE(document_id, user_id)
            );"
        ).expect("Failed to create tables");

        // Migration: add owner_id if missing
        let has_owner = {
            let mut stmt = conn.prepare("PRAGMA table_info(documents)").unwrap();
            let cols: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))
                .unwrap().filter_map(|r| r.ok()).collect();
            cols.contains(&"owner_id".to_string())
        };
        if !has_owner {
            let _ = conn.execute_batch("ALTER TABLE documents ADD COLUMN owner_id TEXT NOT NULL DEFAULT '';");
        }

        // Migration: add dni, cargo, updated_at to users if missing
        let user_cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(users)").unwrap();
            stmt.query_map([], |row| row.get::<_, String>(1))
                .unwrap().filter_map(|r| r.ok()).collect()
        };
        if !user_cols.contains(&"dni".to_string()) {
            let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN dni TEXT;");
        }
        if !user_cols.contains(&"cargo".to_string()) {
            let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN cargo TEXT;");
        }
        if !user_cols.contains(&"active".to_string()) {
            let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN active INTEGER NOT NULL DEFAULT 1;");
        }
        if !user_cols.contains(&"updated_at".to_string()) {
            let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';");
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute("UPDATE users SET updated_at = ?1", params![now]);
        }

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

    fn doc_path(&self, id: &str) -> PathBuf {
        self.data_path.join(format!("{}.bin", id))
    }

    fn pdf_path(&self, id: &str) -> PathBuf {
        self.data_path.join(format!("{}.pdf", id))
    }

    fn file_path(&self, id: &str) -> PathBuf {
        // Try to find actual file by extension
        if let Ok(entries) = std::fs::read_dir(&self.data_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(id) && name != format!("{}.pdf", id) && name != format!("{}.bin", id) {
                    return entry.path();
                }
            }
        }
        // Fallback to .bin
        self.doc_path(id)
    }

    // ---- Users ----

    pub fn authenticate(&self, username: &str, password: &str) -> Option<User> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT id, username, name, dni, cargo FROM users WHERE username = ?1 AND password = ?2 AND active = 1",
            params![username, password],
            |row| Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                name: row.get(2)?,
                dni: row.get(3)?,
                cargo: row.get(4)?,
            }),
        ).ok()
    }

    pub fn search_users(&self, query: &str, exclude_id: &str) -> Vec<UserSearchResult> {
        let db = self.db.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = db.prepare(
            "SELECT id, username, name, dni, cargo FROM users
             WHERE active = 1 AND id != ?1
               AND (name LIKE ?2 OR dni LIKE ?2 OR username LIKE ?2)
             LIMIT 20"
        ).unwrap();
        stmt.query_map(params![exclude_id, pattern], |row| {
            Ok(UserSearchResult {
                id: row.get(0)?,
                username: row.get(1)?,
                name: row.get(2)?,
                dni: row.get(3)?,
                cargo: row.get(4)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    fn seed_users(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();
        let users: [(&str, &str, &str, &str, &str); 5] = [
            ("user1", "Admin123@", "User 1", "12345678", "Director"),
            ("user2", "Admin123@", "User 2", "23456789", "Secretario"),
            ("user3", "Admin123@", "User 3", "34567890", "Analista"),
            ("user4", "Admin123@", "User 4", "45678901", "Asistente"),
            ("user5", "Admin123@", "User 5", "56789012", "Técnico"),
        ];
        for (username, password, name, dni, cargo) in &users {
            let id = uuid::Uuid::new_v4().to_string();
            db.execute(
                "INSERT OR IGNORE INTO users (id, username, password, name, dni, cargo, active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
                params![id, username, password, name, dni, cargo, now],
            ).unwrap_or_default();
        }
        // Update existing users with dni/cargo if missing
        for (i, (_, _, _name, dni, cargo)) in users.iter().enumerate() {
            let uid = format!("user{}", i + 1);
            db.execute(
                "UPDATE users SET dni = COALESCE(dni, ?1), cargo = COALESCE(cargo, ?2), updated_at = ?3 WHERE username = ?4",
                params![dni, cargo, now, uid],
            ).unwrap_or_default();
        }
    }

    // ---- Documents ----

    pub fn create_document(&self, name: &str, ext: &str, editor: &str, owner_id: &str) -> Result<Document, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mime = match ext {
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentationml",
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
            owner_id: owner_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        std::fs::write(self.doc_path(&id), &content).map_err(|e| e.to_string())?;

        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO documents (id, name, ext, mime, editor, size, status, owner_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![doc.id, doc.name, doc.ext, doc.mime, doc.editor,
                    doc.size, doc.status, doc.owner_id, doc.created_at, doc.updated_at],
        ).map_err(|e| e.to_string())?;

        Ok(doc)
    }

    pub fn list_my_documents(&self, user_id: &str) -> Vec<DocumentResponse> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT d.id, d.name, d.ext, d.mime, d.editor, d.size, d.status, d.owner_id,
                    COALESCE(u.name, ''), d.created_at, d.updated_at
             FROM documents d
             LEFT JOIN users u ON u.id = d.owner_id
             WHERE d.owner_id = ?1
             ORDER BY d.updated_at DESC"
        ).unwrap();
        stmt.query_map(params![user_id], |row| {
            Ok(DocumentResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                ext: row.get(2)?,
                mime: row.get(3)?,
                editor: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                owner_id: row.get(7)?,
                owner_name: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                shared: None,
                shared_by: None,
                shared_by_name: None,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn list_shared_documents(&self, user_id: &str) -> Vec<DocumentResponse> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
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
                id: row.get(0)?,
                name: row.get(1)?,
                ext: row.get(2)?,
                mime: row.get(3)?,
                editor: row.get(4)?,
                size: row.get(5)?,
                status: row.get(6)?,
                owner_id: row.get(7)?,
                owner_name: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                shared: Some(true),
                shared_by: Some(row.get(11)?),
                shared_by_name: Some(row.get(12)?),
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    // Kept for backward compat — returns all docs (admin view)
    pub fn list_documents(&self) -> Vec<Document> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT id, name, ext, mime, editor, size, status, owner_id, created_at, updated_at
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
                owner_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn get_document(&self, id: &str) -> Option<Document> {
        let db = self.db.lock().unwrap();
        db.query_row(
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

    pub fn get_document_content(&self, id: &str) -> Option<Vec<u8>> {
        let path = self.file_path(id);
        if !path.exists() { return None; }
        std::fs::read(path).ok()
    }

    pub fn update_document_content(&self, id: &str, content: &[u8]) -> Result<(), String> {
        let path = self.file_path(id);
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();
        db.execute(
            "UPDATE documents SET size = ?1, updated_at = ?2 WHERE id = ?3",
            params![content.len() as u64, now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_document(&self, id: &str, user_id: &str) -> Result<bool, String> {
        let db = self.db.lock().unwrap();

        let is_owner: bool = db.query_row(
            "SELECT COUNT(*) FROM documents WHERE id = ?1 AND owner_id = ?2",
            params![id, user_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        if is_owner {
            let shared_count: i64 = db.query_row(
                "SELECT COUNT(*) FROM document_shares WHERE document_id = ?1",
                params![id],
                |r| r.get(0),
            ).unwrap_or(0);

            if shared_count > 0 {
                // Only remove the owner's implicit share — others still have access
                db.execute("DELETE FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
                    params![id, user_id]).unwrap_or_default();
                Ok(false)
            } else {
                // Last user with access — delete everything
                db.execute("DELETE FROM document_shares WHERE document_id = ?1", params![id]).unwrap_or_default();
                db.execute("DELETE FROM documents WHERE id = ?1", params![id]).unwrap_or_default();
                let _ = std::fs::remove_file(self.doc_path(id));
                let _ = std::fs::remove_file(self.pdf_path(id));
                Ok(true)
            }
        } else {
            // Not owner — just remove the share
            db.execute("DELETE FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
                params![id, user_id]).unwrap_or_default();
            Ok(false)
        }
    }

    pub fn can_access(&self, doc_id: &str, user_id: &str) -> bool {
        let db = self.db.lock().unwrap();
        let is_owner: bool = db.query_row(
            "SELECT COUNT(*) FROM documents WHERE id = ?1 AND owner_id = ?2",
            params![doc_id, user_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if is_owner { return true; }
        let is_shared: bool = db.query_row(
            "SELECT COUNT(*) FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
            params![doc_id, user_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        is_shared
    }

    // ---- PDF ----

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
            params![now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---- Sharing ----

    pub fn share_document(&self, doc_id: &str, user_id: &str, shared_by: &str) -> Result<ShareResponse, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.lock().unwrap();

        let exists: bool = db.query_row(
            "SELECT COUNT(*) FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
            params![doc_id, user_id],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if exists {
            return Err("El documento ya está compartido con este usuario".to_string());
        }

        let owner_id: String = db.query_row(
            "SELECT owner_id FROM documents WHERE id = ?1",
            params![doc_id],
            |r| r.get(0),
        ).map_err(|_| "Documento no encontrado".to_string())?;

        if owner_id == user_id {
            return Err("No puedes compartir un documento contigo mismo".to_string());
        }

        db.execute(
            "INSERT INTO document_shares (id, document_id, user_id, shared_by, permission, created_at)
             VALUES (?1, ?2, ?3, ?4, 'edit', ?5)",
            params![id, doc_id, user_id, shared_by, now],
        ).map_err(|e| format!("Error al compartir: {}", e))?;

        let user_name: String = db.query_row(
            "SELECT name FROM users WHERE id = ?1",
            params![user_id],
            |r| r.get(0),
        ).unwrap_or_default();
        let shared_by_name: String = db.query_row(
            "SELECT name FROM users WHERE id = ?1",
            params![shared_by],
            |r| r.get(0),
        ).unwrap_or_default();

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

    pub fn get_shares(&self, doc_id: &str) -> Vec<ShareResponse> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
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
                id: row.get(0)?,
                document_id: row.get(1)?,
                user_id: row.get(2)?,
                user_name: row.get(3)?,
                shared_by: row.get(4)?,
                shared_by_name: row.get(5)?,
                permission: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn remove_share(&self, doc_id: &str, user_id: &str) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let affected = db.execute(
            "DELETE FROM document_shares WHERE document_id = ?1 AND user_id = ?2",
            params![doc_id, user_id],
        ).map_err(|e| format!("Error al eliminar: {}", e))?;
        if affected == 0 {
            Err("No se encontró el compartido".to_string())
        } else {
            Ok(())
        }
    }
}
