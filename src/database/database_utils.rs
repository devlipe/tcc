use crate::SQLiteConnector;
use anyhow::Result;

pub fn create_did_table(sqlite: &SQLiteConnector) -> Result<usize> {
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

pub fn create_vc_table(sqlite: &SQLiteConnector) -> Result<usize> {
    let sql_query = r#"
        CREATE TABLE IF NOT EXISTS vcs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, 
                vc TEXT NOT NULL,                                      
                type TEXT NOT NULL,
                issuer INTEGER NOT NULL,
                holder INTEGER NOT NULL,
                FOREIGN KEY (issuer) REFERENCES dids(id),
                FOREIGN KEY (holder) REFERENCES dids(id)                                            
        )"#;
    
    sqlite.execute(sql_query, [])
}

pub fn create_database_tables(sqlite: &SQLiteConnector) -> Result<()> {
    create_did_table(sqlite)?;
    create_vc_table(sqlite)?;
    Ok(())
}
    