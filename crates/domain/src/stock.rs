use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub market: String,
    pub sector: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStock {
    pub code: String,
    pub name: String,
    pub market: Option<String>,
    pub sector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSearchQuery {
    pub query: String,
    pub limit: Option<i32>,
}

impl StockSearchQuery {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(10).min(50).max(1)
    }
}
