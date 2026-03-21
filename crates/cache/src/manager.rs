use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use anyhow::Result;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub cached_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl<T> CacheEntry<T> {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheManager {
    base_path: PathBuf,
    default_ttl: Option<Duration>,
}

impl CacheManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            default_ttl: Some(Duration::from_secs(3600)),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    pub fn without_ttl(mut self) -> Self {
        self.default_ttl = None;
        self
    }

    fn ensure_dir(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                debug!("Created cache directory: {:?}", parent);
            }
        }
        Ok(())
    }

    fn get_cache_path(&self, category: &str, key: &str) -> PathBuf {
        self.base_path.join(category).join(format!("{}.json", key))
    }

    pub fn get<T: DeserializeOwned>(&self, category: &str, key: &str) -> Result<Option<T>> {
        let path = self.get_cache_path(category, key);
        
        if !path.exists() {
            debug!("Cache miss: {}/{}", category, key);
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        let entry: CacheEntry<T> = serde_json::from_str(&content)?;

        if entry.is_expired() {
            info!("Cache expired: {}/{}", category, key);
            fs::remove_file(&path)?;
            return Ok(None);
        }

        debug!("Cache hit: {}/{}", category, key);
        Ok(Some(entry.data))
    }

    pub fn set<T: Serialize + ?Sized>(&self, category: &str, key: &str, data: &T) -> Result<()> {
        let path = self.get_cache_path(category, key);
        self.ensure_dir(&path)?;

        let now = Utc::now();
        let expires_at = self.default_ttl.and_then(|ttl| {
            (now + chrono::Duration::from_std(ttl).unwrap()).into()
        });

        let entry = CacheEntry {
            data: serde_json::to_value(data)?,
            cached_at: now,
            expires_at,
        };
        let content = serde_json::to_string_pretty(&entry)?;

        fs::write(&path, content)?;
        debug!("Cache set: {}/{}", category, key);
        Ok(())
    }

    pub fn set_with_ttl<T: Serialize + ?Sized>(
        &self,
        category: &str,
        key: &str,
        data: &T,
        ttl: Duration,
    ) -> Result<()> {
        let path = self.get_cache_path(category, key);
        self.ensure_dir(&path)?;

        let now = Utc::now();
        let expires_at = now + chrono::Duration::from_std(ttl).unwrap();
        
        let entry = serde_json::json!({
            "data": data,
            "cached_at": now,
            "expires_at": expires_at
        });
        let content = serde_json::to_string_pretty(&entry)?;

        fs::write(&path, content)?;
        debug!("Cache set with TTL: {}/{}", category, key);
        Ok(())
    }

    pub fn remove(&self, category: &str, key: &str) -> Result<bool> {
        let path = self.get_cache_path(category, key);
        
        if path.exists() {
            fs::remove_file(&path)?;
            debug!("Cache removed: {}/{}", category, key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_category(&self, category: &str) -> Result<usize> {
        let dir = self.base_path.join(category);
        
        if !dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                fs::remove_file(entry.path())?;
                count += 1;
            }
        }

        info!("Cleared {} cache entries in {}", count, category);
        Ok(count)
    }

    pub fn clear_all(&self) -> Result<usize> {
        let mut total = 0;
        
        if !self.base_path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let category = entry.file_name().to_string_lossy().to_string();
                total += self.clear_category(&category)?;
            }
        }

        info!("Cleared all cache: {} entries", total);
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_set_and_get() {
        let dir = tempdir().unwrap();
        let cache = CacheManager::new(dir.path()).without_ttl();

        cache.set("test", "key1", &"value1").unwrap();
        let result: Option<String> = cache.get("test", "key1").unwrap();
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let dir = tempdir().unwrap();
        let cache = CacheManager::new(dir.path());

        let result: Option<String> = cache.get("test", "nonexistent").unwrap();
        assert!(result.is_none());
    }
}
