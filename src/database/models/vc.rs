use crate::Did;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default)]
pub struct Vc {
    id: i64,
    vc: String,
    tp: String,
    issuer: Did,
    holder: Did,
    created_at: NaiveDateTime,
}

impl Vc {
    pub fn new(
        id: i64,
        vc: String,
        tp: String,
        issuer: Did,
        holder: Did,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            vc,
            tp,
            issuer,
            holder,
            created_at,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn vc(&self) -> &str {
        &self.vc
    }

    pub fn tp(&self) -> &str {
        &self.tp
    }

    pub fn issuer(&self) -> &Did {
        &self.issuer
    }

    pub fn holder(&self) -> &Did {
        &self.holder
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}
