use std::fmt::Debug;
use chrono::naive::NaiveDateTime;

#[derive(Debug)]
pub struct Did {
    id: i32,
    did: String,
    fragment: String,
    name: String,
    created_at: NaiveDateTime,
}

impl Did {
    pub fn new(id: i32, did: String, fragment: String, name: String, created_at: NaiveDateTime) -> Self {
        Self {
            id,
            did,
            fragment,
            name,
            created_at,
        }
    }

    pub fn id(&self) -> i32 {
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
}
