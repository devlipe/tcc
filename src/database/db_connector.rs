use anyhow::Result;
use identity_iota::iota::IotaDocument;
use crate::{Did, Vc};

pub trait DBConnector {

    fn save_did_document(&self, did: &IotaDocument, owner: &String) -> Result<usize> ;

    fn get_did_from_id(&self, id: i64) -> Result<Did>;
    
    fn get_stored_dids(&self) -> Result<Vec<Did>>;
    
    fn save_vc(&self, vc: &str, issuer: i64, holder: i64, tp: &String) -> Result<usize>;
    
    fn get_vc_from_id(&self, id: i64) -> Result<Vc>;
    
    fn get_stored_vcs(&self) -> Result<Vec<Vc>>;
}