use sqlx::PgPool;
use crate::scheduler::SyncScheduler;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub scheduler: SyncScheduler,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        let scheduler = SyncScheduler::new(db.clone());
        Self { db, scheduler }
    }
    
    pub async fn start_scheduler(&self) {
        self.scheduler.start().await;
    }
}
