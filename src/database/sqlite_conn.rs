use crate::{extract_kid, DBConnector, Did, Vc};
use anyhow::Error;
use anyhow::Result;
use chrono::NaiveDateTime;
use identity_iota::iota::IotaDocument;
use rusqlite::{Connection, Params, Row};

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

    pub fn execute<P: Params>(&self, query: &str, params: P) -> Result<usize> {
        let result = self.conn.execute(query, params);

        match result {
            Ok(n) => Ok(n),
            Err(e) => Err(e.into()),
        }
    }

    fn build_did_model(row: &Row) -> Result<Did, Error> {
        let created_at: String = row.get(4)?;
        Ok(Did::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")?,
        ))
    }

    fn build_vc_model(&self, row: &Row) -> Result<Vc, Error> {
        let created_at: String = row.get(5)?;
        Ok(Vc::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            self.get_did_from_id(row.get(3)?).unwrap_or_default(),
            self.get_did_from_id(row.get(4)?).unwrap_or_default(),
            NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")?,
        ))
    }
}

impl DBConnector for SQLiteConnector {
    fn save_did_document(&self, did: &IotaDocument, owner: &String) -> Result<usize, Error> {
        // The "dids" table has the following columns:
        // - id: INTEGER PRIMARY KEY AUTOINCREMENT
        // - created_at: TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        // - did: TEXT NOT NULL
        // - fragment: TEXT

        let sql_query = r#"
            INSERT INTO dids (did, fragment, name, created_at)
            VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
        "#;

        let fragment = extract_kid(did)?;

        self.execute(sql_query, [did.id().to_string(), fragment, owner.clone()])
            .map_err(|e| e.into())
    }

    fn get_did_from_id(&self, id: i64) -> Result<Did> {
        let sql_query = r#"
           SELECT id, did, fragment, name, created_at FROM dids WHERE id = ?1
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;
        let mut rows = stmt.query([id])?;

        let row = rows.next()?;

        if let Some(row) = row {
            Self::build_did_model(row)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows.into())
        }
    }

    fn get_stored_dids(&self) -> Result<Vec<Did>> {
        let sql_query = r#"
            SELECT id, did, fragment, name, created_at FROM dids
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;

        let did_iter = stmt
            .query_map([], |row| Ok(Self::build_did_model(row).unwrap()))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(did_iter)
    }

    fn save_vc(&self, vc: &str, issuer: i64, holder: i64, tp: &String) -> Result<usize> {
        let sql_query = r#"
            INSERT INTO vcs (vc, type, issuer, holder, created_at)
            VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
        "#;

        self.execute(
            sql_query,
            [vc, tp, &issuer.to_string(), &holder.to_string()],
        )
        .map_err(|e| e.into())
    }

    fn get_vc_from_id(&self, id: i64) -> Result<Vc> {
        let sql_query = r#"
            SELECT id, vc, type, issuer, holder, created_at FROM vcs WHERE id = ?1
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;
        let mut rows = stmt.query([id])?;

        let row = rows.next()?;

        if let Some(row) = row {
            Ok(self.build_vc_model(row)?)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows.into())
        }
    }

    fn get_stored_vcs(&self) -> Result<Vec<Vc>> {
        let sql_query = r#"
            SELECT vcs.id, vc, type, issuer, holder, vcs.created_at
            FROM
                vcs
            INNER JOIN
                dids AS issuer_did ON vcs.issuer = issuer_did.id
            INNER JOIN
                dids AS holder_did ON vcs.holder = holder_did.id
        "#;

        let mut stmt = self.conn.prepare(sql_query)?;

        let vc_iter = stmt
            .query_map([], |row| Ok(self.build_vc_model(row).unwrap()))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(vc_iter)
    }
}

// Implement Default for SQLiteConnector
impl Default for SQLiteConnector {
    fn default() -> Self {
        // Use new() with an empty string to create an in-memory database connection
        SQLiteConnector::new("").expect("Failed to create in-memory SQLite connection")
    }
}
