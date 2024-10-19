use anyhow::Result;
use identity_iota::iota::IotaDocument;
use crate::Did;

pub trait DBConnector {

    fn save_did_document(&self, did: &IotaDocument, owner: String) -> Result<usize> ;

    fn get_did_from_id(&self, id: i64) -> rusqlite::Result<String>;
    
    fn get_stored_dids(&self) -> rusqlite::Result<Vec<Did>>;
}