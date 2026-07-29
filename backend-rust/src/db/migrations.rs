use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) {
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

    // Migration: add owner_id
    let has_owner = {
        let mut stmt = conn.prepare("PRAGMA table_info(documents)").unwrap();
        let cols: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap().filter_map(|r| r.ok()).collect();
        cols.contains(&"owner_id".to_string())
    };
    if !has_owner {
        let _ = conn.execute_batch("ALTER TABLE documents ADD COLUMN owner_id TEXT NOT NULL DEFAULT '';");
    }

    // Migration: add user columns
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
        let _ = conn.execute("UPDATE users SET updated_at = ?1", rusqlite::params![now]);
    }
}
