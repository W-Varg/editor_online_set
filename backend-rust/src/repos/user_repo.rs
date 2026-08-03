use rusqlite::params;
use crate::db::DbConn;
use crate::dto::UserSearchResult;
use crate::models::User;

pub fn get_by_id(db: &DbConn, user_id: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, username, name, dni, cargo, email FROM users WHERE id = ?1",
        params![user_id],
        |row| Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            name: row.get(2)?,
            dni: row.get(3)?,
            cargo: row.get(4)?,
            email: row.get(5)?,
        }),
    ).ok()
}

pub fn authenticate(db: &DbConn, username: &str, password: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT id, username, name, dni, cargo, email FROM users WHERE username = ?1 AND password = ?2 AND active = 1",
        params![username, password],
        |row| Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            name: row.get(2)?,
            dni: row.get(3)?,
            cargo: row.get(4)?,
            email: row.get(5)?,
        }),
    ).ok()
}

pub fn list(db: &DbConn) -> Vec<UserSearchResult> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, username, name, dni, cargo FROM users
         WHERE active = 1
         ORDER BY name COLLATE NOCASE"
    ).unwrap();
    stmt.query_map([], |row| {
        Ok(UserSearchResult {
            id: row.get(0)?,
            username: row.get(1)?,
            name: row.get(2)?,
            dni: row.get(3)?,
            cargo: row.get(4)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect()
}

pub fn search(db: &DbConn, query: &str, exclude_id: &str) -> Vec<UserSearchResult> {
    let conn = db.lock().unwrap();
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
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

pub fn search_for_document(
    db: &DbConn,
    doc_id: &str,
    query: &str,
    exclude_id: &str,
) -> (Vec<UserSearchResult>, Vec<UserSearchResult>) {
    let conn = db.lock().unwrap();
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.name, u.dni, u.cargo, s.user_id
         FROM users u
         LEFT JOIN document_shares s ON s.document_id = ?1 AND s.user_id = u.id
         WHERE u.active = 1 AND u.id != ?2
           AND (?3 = '%' OR u.name LIKE ?3 OR u.dni LIKE ?3 OR u.username LIKE ?3)
         ORDER BY u.name COLLATE NOCASE
         LIMIT 50"
    ).unwrap();

    let mut shared = Vec::new();
    let mut found = Vec::new();
    let rows = stmt.query_map(rusqlite::params![doc_id, exclude_id, pattern], |row| {
        Ok((UserSearchResult {
            id: row.get(0)?,
            username: row.get(1)?,
            name: row.get(2)?,
            dni: row.get(3)?,
            cargo: row.get(4)?,
        }, row.get::<_, Option<String>>(5)?.is_some()))
    }).unwrap();

    for row in rows.flatten() {
        if row.1 { shared.push(row.0); } else { found.push(row.0); }
    }
    (shared, found)
}

pub fn seed(db: &DbConn) {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.lock().unwrap();
    let users: [(&str, &str, &str, &str, &str, &str); 5] = [
        ("user1", "Admin123@", "User 1", "12345678", "Director", "user1@empresa.local"),
        ("user2", "Admin123@", "User 2", "23456789", "Secretario", "user2@empresa.local"),
        ("user3", "Admin123@", "User 3", "34567890", "Analista", "user3@empresa.local"),
        ("user4", "Admin123@", "User 4", "45678901", "Asistente", "user4@empresa.local"),
        ("user5", "Admin123@", "User 5", "56789012", "Técnico", "user5@empresa.local"),
    ];
    for (username, password, name, dni, cargo, email) in &users {
        // ID determinista por username (UUID v5) para que los reseeds no
        // regeneren IDs y los documentos nunca queden con owner_id huérfanos.
        let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, username.as_bytes()).to_string();
        conn.execute(
            "INSERT OR IGNORE INTO users (id, username, password, name, dni, cargo, email, active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
            params![id, username, password, name, dni, cargo, email, now],
        ).unwrap_or_default();
    }
    for (i, (_, _, _name, dni, cargo, email)) in users.iter().enumerate() {
        let uid = format!("user{}", i + 1);
        conn.execute(
            "UPDATE users SET dni = COALESCE(dni, ?1), cargo = COALESCE(cargo, ?2), email = COALESCE(email, ?3), updated_at = ?4 WHERE username = ?5",
            params![dni, cargo, email, now, uid],
        ).unwrap_or_default();
    }
}
