use std::time::Duration;
use anyhow::Result;
use crate::CacheManager;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SectorInfo {
    pub name: String,
    pub stock_count: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct SectorCache {
    cache: CacheManager,
}

impl SectorCache {
    pub fn new(base_path: &std::path::Path) -> Self {
        Self {
            cache: CacheManager::new(base_path.join("sectors"))
                .with_ttl(Duration::from_secs(86400)),
        }
    }

    pub fn get_sector(&self, name: &str) -> Result<Option<SectorInfo>> {
        self.cache.get("info", name)
    }

    pub fn set_sector(&self, info: &SectorInfo) -> Result<()> {
        self.cache.set("info", &info.name, info)
    }

    pub fn get_all_sectors(&self) -> Result<Option<Vec<SectorInfo>>> {
        self.cache.get("all", "sectors")
    }

    pub fn set_all_sectors(&self, sectors: &[SectorInfo]) -> Result<()> {
        self.cache.set("all", "sectors", sectors)
    }

    pub fn invalidate(&self) -> Result<usize> {
        self.cache.clear_all()
    }
}
