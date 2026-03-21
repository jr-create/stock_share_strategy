use std::time::Duration;
use anyhow::Result;
use domain::Stock;
use crate::CacheManager;

pub struct StockCache {
    cache: CacheManager,
}

impl StockCache {
    pub fn new(base_path: &std::path::Path) -> Self {
        Self {
            cache: CacheManager::new(base_path.join("stocks"))
                .with_ttl(Duration::from_secs(86400)),
        }
    }

    pub fn get_stock(&self, code: &str) -> Result<Option<Stock>> {
        self.cache.get("stocks", code)
    }

    pub fn set_stock(&self, stock: &Stock) -> Result<()> {
        self.cache.set("stocks", &stock.code, stock)
    }

    pub fn get_stocks_by_sector(&self, sector: &str) -> Result<Option<Vec<Stock>>> {
        self.cache.get("sectors", sector)
    }

    pub fn set_stocks_by_sector(&self, sector: &str, stocks: &[Stock]) -> Result<()> {
        self.cache.set("sectors", sector, stocks)
    }

    pub fn get_all_stocks(&self) -> Result<Option<Vec<Stock>>> {
        self.cache.get("all", "stocks")
    }

    pub fn set_all_stocks(&self, stocks: &[Stock]) -> Result<()> {
        self.cache.set("all", "stocks", stocks)
    }

    pub fn invalidate_stock(&self, code: &str) -> Result<bool> {
        self.cache.remove("stocks", code)
    }

    pub fn invalidate_all(&self) -> Result<usize> {
        self.cache.clear_all()
    }
}
