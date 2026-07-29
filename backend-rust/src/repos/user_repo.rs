use rusqlite::params;
use crate::db::DbConn;
use crate::dto::UserSearchResult;
use crate::models::User;

pub fn authenticate(db: &DbConn, username: &str, password: &str) -> Option<User> {
    let conn = db.lock().unwrap();
    conn.query_row(
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

pub fn seed(db: &DbConn) {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.lock().unwrap();
    let users: [(&str, &str, &str, &str, &str); 5] = [
        ("user1", "Admin123@", "User 1", "12345678", "Director"),
        ("user2", "Admin123@", "User 2", "23456789", "Secretario"),
        ("user3", "Admin123@", "User 3", "34567890", "Analista"),
        ("user4", "Admin123@", "User 4", "45678901", "Asistente"),
        ("user5", "Admin123@", "User 5", "56789012", "Técnico"),
    ];
    for (username, password, name, dni, cargo) in &users {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT OR IGNORE INTO users (id, username, password, name, dni, cargo, active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
            params![id, username, password, name, dni, cargo, now],
        ).unwrap_or_default();
    }
    for (i, (_, _, _name, dni, cargo)) in users.iter().enumerate() {
        let uid = format!("user{}", i + 1);
        conn.execute(
            "UPDATE users SET dni = COALESCE(dni, ?1), cargo = COALESCE(cargo, ?2), updated_at = ?3 WHERE username = ?4",
            params![dni, cargo, now, uid],
        ).unwrap_or_default();
    }
}
