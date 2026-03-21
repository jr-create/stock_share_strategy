use std::sync::Arc;
use sqlx::PgPool;
use crate::scheduler::SyncScheduler;
use crate::middleware::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub scheduler: SyncScheduler,
    pub rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        let scheduler = SyncScheduler::new(db.clone());
        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));
        Self { db, scheduler, rate_limiter }
    }
    
    pub async fn start_scheduler(&self) {
        self.scheduler.start().await;
    }
}
