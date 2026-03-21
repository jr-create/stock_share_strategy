use sqlx::PgPool;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Creating database connection pool...");
    PgPool::connect(database_url).await
}

pub async fn create_pool_with_max_connections(
    database_url: &str,
    max_connections: u32,
) -> Result<PgPool, sqlx::Error> {
    info!("Creating database connection pool with max {} connections", max_connections);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    info!("Running database migrations...");
    sqlx::migrate!("../../migrations").run(pool).await
}

pub async fn health_check(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await;
    Ok(result.is_ok())
}
