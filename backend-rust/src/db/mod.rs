pub mod migrations;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub type DbConn = Arc<Mutex<Connection>>;

pub fn open_connection(data_dir: &str) -> (DbConn, PathBuf) {
    let data_path = PathBuf::from(data_dir);
    std::fs::create_dir_all(&data_path).unwrap_or_default();
    let db_path = data_path.join("editor.db");
    let conn = Connection::open(&db_path).expect("Failed to open SQLite database");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap_or_default();
    (Arc::new(Mutex::new(conn)), data_path)
}
