use crate::SQLiteConnector;

pub fn create_did_table(sqlite: &SQLiteConnector) -> rusqlite::Result<usize> {
    let sql_query = r#"
        CREATE TABLE IF NOT EXISTS dids (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, 
                did TEXT NOT NULL,                                      
                fragment TEXT,
                name TEXT                                            
        )"#;
    
    
    sqlite.execute(sql_query, [])
}

pub fn create_tables(sqlite: &SQLiteConnector) -> rusqlite::Result<()> {
    create_did_table(sqlite)?;
    Ok(())
}
    