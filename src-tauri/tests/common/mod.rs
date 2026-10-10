use rusqlite::Connection;

pub fn open_clean_database() -> rusqlite::Result<Connection> {
    Connection::open_in_memory()
}
