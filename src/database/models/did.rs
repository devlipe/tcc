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
}
