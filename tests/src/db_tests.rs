use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_database_connection() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/stock_replay?client_encoding=UTF8".to_string());
    
    println!("连接数据库: {}", db_url);
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(result.0, 1);
    println!("✅ 数据库连接成功！");
    
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn test_check_tables() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/stock_replay?client_encoding=UTF8".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'")
        .fetch_one(&pool)
        .await?;
    
    println!("数据库中的表数量: {}", result.0);
    
    pool.close().await;
    Ok(())
}
