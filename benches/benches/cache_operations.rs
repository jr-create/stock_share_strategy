use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cache::{CacheManager, StockCache, SectorCache, SectorInfo};
use domain::Stock;
use tempfile::tempdir;

fn bench_cache_manager(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let cache = CacheManager::new(dir.path()).without_ttl();

    c.bench_function("cache_set", |b| {
        b.iter(|| cache.set("test", "key", &black_box("value")))
    });

    cache.set("test", "key", &"value").unwrap();
    c.bench_function("cache_get", |b| {
        b.iter(|| {
            let _: Option<String> = cache.get("test", "key").unwrap();
        })
    });
}

fn bench_stock_cache(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let cache = StockCache::new(dir.path());

    let stock = Stock {
        code: "000001".to_string(),
        name: "平安银行".to_string(),
        market: "SZ".to_string(),
        sector: "银行".to_string(),
        updated_at: chrono::Utc::now(),
    };

    c.bench_function("stock_cache_set", |b| {
        b.iter(|| cache.set_stock(black_box(&stock)))
    });

    cache.set_stock(&stock).unwrap();
    c.bench_function("stock_cache_get", |b| {
        b.iter(|| cache.get_stock(black_box("000001")))
    });
}

fn bench_sector_cache(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let cache = SectorCache::new(dir.path());

    let sector = SectorInfo {
        name: "银行".to_string(),
        stock_count: 42,
        updated_at: chrono::Utc::now(),
    };

    c.bench_function("sector_cache_set", |b| {
        b.iter(|| cache.set_sector(black_box(&sector)))
    });

    cache.set_sector(&sector).unwrap();
    c.bench_function("sector_cache_get", |b| {
        b.iter(|| cache.get_sector(black_box("银行")))
    });
}

criterion_group!(cache_manager, bench_cache_manager);
criterion_group!(stock_cache, bench_stock_cache);
criterion_group!(sector_cache, bench_sector_cache);
criterion_main!(cache_manager, stock_cache, sector_cache);
