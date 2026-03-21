use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Replay {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub trade_date: NaiveDate,
    pub market_sentiment: String,
    pub market_view: String,
    pub sector_focus: String,
    pub watchlist: String,
    pub plan: String,
    pub execution: String,
    pub review: String,
    pub is_public: bool,
    pub expiry_date: Option<NaiveDate>,
    pub is_prediction: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReplay {
    pub title: String,
    pub trade_date: NaiveDate,
    pub market_sentiment: String,
    pub market_view: String,
    pub sector_focus: String,
    pub watchlist: String,
    pub plan: String,
    pub execution: String,
    pub review: String,
    pub is_public: bool,
    pub expiry_date: Option<NaiveDate>,
    pub is_prediction: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateReplay {
    pub title: Option<String>,
    pub trade_date: Option<NaiveDate>,
    pub market_sentiment: Option<String>,
    pub market_view: Option<String>,
    pub sector_focus: Option<String>,
    pub watchlist: Option<String>,
    pub plan: Option<String>,
    pub execution: Option<String>,
    pub review: Option<String>,
    pub is_public: Option<bool>,
    pub expiry_date: Option<NaiveDate>,
    pub is_prediction: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayWithAuthor {
    pub replay: Replay,
    pub author_username: String,
    pub author_avatar: String,
    pub like_count: i64,
    pub comment_count: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayFilter {
    pub user_id: Option<i64>,
    pub tag: Option<String>,
    pub stock_code: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_public: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

impl ReplayFilter {
    pub fn page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).min(100).max(1)
    }

    pub fn offset(&self) -> i32 {
        (self.page() - 1) * self.per_page()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub total_pages: i32,
    pub current_page: i32,
    pub per_page: i32,
}

impl PaginationMeta {
    pub fn new(total: i64, current_page: i32, per_page: i32) -> Self {
        let total_pages = if per_page > 0 {
            ((total as f64) / (per_page as f64)).ceil() as i32
        } else {
            0
        };
        Self {
            total,
            total_pages,
            current_page,
            per_page,
        }
    }

    pub fn has_prev(&self) -> bool {
        self.current_page > 1
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_filter_defaults() {
        let filter = ReplayFilter::default();
        assert_eq!(filter.page(), 1);
        assert_eq!(filter.per_page(), 20);
        assert_eq!(filter.offset(), 0);
    }

    #[test]
    fn test_replay_filter_pagination() {
        let filter = ReplayFilter {
            page: Some(3),
            per_page: Some(10),
            ..Default::default()
        };
        assert_eq!(filter.page(), 3);
        assert_eq!(filter.per_page(), 10);
        assert_eq!(filter.offset(), 20);
    }

    #[test]
    fn test_replay_filter_bounds() {
        let filter = ReplayFilter {
            page: Some(0),
            per_page: Some(200),
            ..Default::default()
        };
        assert_eq!(filter.page(), 1);
        assert_eq!(filter.per_page(), 100);
    }
}
