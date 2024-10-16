use rusqlite::{Connection, Result};

// Define a struct to represent the SQLite database connection
pub struct SQLiteConnector {
    conn: Connection,
}

// Implement the constructor for SQLiteConnector
impl SQLiteConnector {
    pub fn new(conn_str: &str) -> Result<Self, anyhow::Error> {
        let conn = if conn_str.is_empty() {
            let conn = Connection::open_in_memory()?;
            conn
        } else {
            let conn = Connection::open(conn_str)?;
            conn
        };

        Ok(Self { conn })
    }
    
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
    
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
    
    
}

