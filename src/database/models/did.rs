use std::fmt::Debug;
use chrono::naive::NaiveDateTime;
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

impl Debug for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Did: id: {}, did: {}, fragment: {}, name: {}, created_at: {}", self.id, self.did, self.fragment, self.name, self.created_at)
    }
}