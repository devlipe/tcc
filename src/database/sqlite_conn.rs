use crate::{extract_kid, DBConnector, Did};
use anyhow::Error;
use chrono::NaiveDateTime;
use identity_iota::iota::IotaDocument;
use rusqlite::{Connection, Params, Result};

// Define a struct to represent the SQLite database connection
pub struct SQLiteConnector {
    conn: Connection,
}


// Implement the constructor for SQLiteConnector
impl SQLiteConnector {
    pub fn new(conn_str: &str) -> Result<Self, Error> {
        let conn = if conn_str.is_empty() {
            let conn = Connection::open_in_memory()?;
            conn
        } else {
            let conn = Connection::open(conn_str)?;
            conn
        };

        Ok(Self { conn })
    }
    
    pub fn execute<P:Params>(&self, query: &str, params: P) -> Result<usize> {
        self.conn.execute(query, params)
    }
}

impl DBConnector for SQLiteConnector{
    fn save_did_document(&self, did: &IotaDocument, owner: String) -> Result<usize, Error> {
        // The "dids" table has the following columns:
        // - id: INTEGER PRIMARY KEY AUTOINCREMENT
        // - created_at: TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        // - did: TEXT NOT NULL
        // - fragment: TEXT
        
        let sql_query = r#"
            INSERT INTO dids (did, fragment, name, created_at)
            VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
        "#;
        
        let fragment =  extract_kid(did)?;
        
        self.execute(sql_query, [did.id().to_string(), fragment, owner]).map_err(|e| e.into())
    }

    fn get_did_from_id(&self, id: i64) -> Result<String> {
        let sql_query = r#"
            SELECT did FROM dids WHERE id = ?1
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;
        let mut rows = stmt.query([id])?;

        let row = rows.next()?;

        if let Some(row) = row {
            let did: String = row.get(0)?;
            Ok(did)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows.into())
        }
    }

    fn get_stored_dids(&self) -> Result<Vec<Did>> {
        let sql_query = r#"
            SELECT id, did, fragment, name, created_at FROM dids
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;

        let did_iter = stmt.query_map([], |row| {
            let created_at: String = row.get(4)?;
            Ok(Did::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S").unwrap(),
            ))
        })?;
        did_iter.collect()
    }
    
}

// Implement Default for SQLiteConnector
impl Default for SQLiteConnector {
    fn default() -> Self {
        // Use new() with an empty string to create an in-memory database connection
        SQLiteConnector::new("").expect("Failed to create in-memory SQLite connection")
    }
}

