use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{interval, Duration};
use tracing::{info, error};

#[derive(Clone)]
pub struct SyncScheduler {
    db: PgPool,
    running: Arc<AtomicBool>,
}

impl SyncScheduler {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub async fn start(&self) {
        let db = self.db.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60 * 60 * 24));
            
            loop {
                interval.tick().await;
                
                if running.load(Ordering::Relaxed) {
                    info!("开始执行定时股票数据同步...");
                    if let Err(e) = Self::sync_stocks(&db).await {
                        error!("股票数据同步失败: {}", e);
                    }
                }
            }
        });
    }
    
    pub async fn trigger_sync(&self) -> Result<usize> {
        Self::sync_stocks(&self.db).await
    }
    
    async fn sync_stocks(db: &PgPool) -> Result<usize> {
        use std::process::Command;
        
        info!("执行股票数据同步脚本...");
        
        let output = Command::new("python")
            .arg("scripts/akshare_data.py")
            .arg("--sync-db")
            .output()
            .map_err(|e| anyhow::anyhow!("执行脚本失败: {}", e))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("同步输出: {}", stdout);
            Ok(0)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("脚本执行错误: {}", stderr))
        }
    }
    
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Relaxed);
    }
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
