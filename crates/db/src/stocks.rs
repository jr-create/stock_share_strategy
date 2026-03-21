use domain::{Stock, NewStock};
use sqlx::PgPool;
use anyhow::Result;

pub async fn create(pool: &PgPool, new_stock: &NewStock) -> Result<Stock> {
    let stock = sqlx::query_as::<_, Stock>(
        r#"
        INSERT INTO stocks (code, name, market, sector)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (code) DO UPDATE SET
            name = EXCLUDED.name,
            market = COALESCE(EXCLUDED.market, stocks.market),
            sector = COALESCE(EXCLUDED.sector, stocks.sector)
        RETURNING *
        "#
    )
    .bind(&new_stock.code)
    .bind(&new_stock.name)
    .bind(&new_stock.market)
    .bind(&new_stock.sector)
    .fetch_one(pool)
    .await?;

    Ok(stock)
}

pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Option<Stock>> {
    let stock = sqlx::query_as::<_, Stock>(
        "SELECT * FROM stocks WHERE code = $1"
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    Ok(stock)
}

pub async fn search(pool: &PgPool, query: &str, limit: i32) -> Result<Vec<Stock>> {
    let pattern = format!("{}%", query);
    
    let stocks = sqlx::query_as::<_, Stock>(
        r#"
        SELECT * FROM stocks
        WHERE code ILIKE $1 
           OR name ILIKE $1
           OR SUBSTRING(code, 3) ILIKE $1
        ORDER BY code
        LIMIT $2
        "#
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(stocks)
}

pub async fn list_by_sector(pool: &PgPool, sector: &str) -> Result<Vec<Stock>> {
    let stocks = sqlx::query_as::<_, Stock>(
        "SELECT * FROM stocks WHERE sector = $1 ORDER BY code"
    )
    .bind(sector)
    .fetch_all(pool)
    .await?;

    Ok(stocks)
}
