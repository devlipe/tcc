use std::fmt::Debug;
use chrono::naive::NaiveDateTime;
use identity_iota::iota::{IotaDID, IotaDocument};
use identity_iota::prelude::Resolver;


#[derive(Debug, Clone)]
pub struct Did {
    id: i64,
    did: String,
    fragment: String,
    name: String,
    created_at: NaiveDateTime,
}

impl Did {
    pub fn new(id: i64, did: String, fragment: String, name: String, created_at: NaiveDateTime) -> Self {
        Self {
            id,
            did,
            fragment,
            name,
            created_at,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn did(&self) -> &str {
        &self.did
    }

    pub fn fragment(&self) -> &str {
        &self.fragment
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    pub async fn resolve_to_iota_document(&self, resolver : &Resolver<IotaDocument>) -> IotaDocument {
        let did = IotaDID::parse(&self.did).unwrap();
        resolver.resolve(&did).await.unwrap()

    }
}
