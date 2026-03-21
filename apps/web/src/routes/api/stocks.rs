use axum::{
    extract::{State, Query, Path},
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i32>,
}

async fn search_stocks(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<domain::Stock>>> {
    let limit = query.limit.unwrap_or(10).min(50);
    let stocks = db::stocks::search(&state.db, &query.q, limit).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(stocks))
}

#[derive(Debug, Serialize, Deserialize)]
struct StockQuote {
    code: String,
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
    open: f64,
    high: f64,
    low: f64,
    prev_close: f64,
    volume: i64,
    amount: f64,
    turnover_rate: f64,
    pe: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockQuote {
    code: String,
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
    open: f64,
    high: f64,
    low: f64,
    prev_close: f64,
    volume: i64,
    amount: f64,
    turnover_rate: f64,
    pe: f64,
}

async fn get_stock_quote(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<MockQuote>> {
    let stock = db::stocks::find_by_code(&state.db, &code).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Stock {} not found", code)))?;
    
    let hash: f64 = code.chars().map(|c| c as u32 as f64).sum();
    let hash_i64: i64 = code.chars().map(|c| c as i64).sum();
    
    let quote = MockQuote {
        code: stock.code.clone(),
        name: stock.name.clone(),
        price: 10.0 + (hash % 100.0),
        change: (hash % 10.0) - 5.0,
        change_percent: ((hash % 10.0) - 5.0) / 10.0 * 100.0,
        open: 10.0 + (hash % 100.0) - 0.5,
        high: 10.0 + (hash % 100.0) + 0.5,
        low: 10.0 + (hash % 100.0) - 1.0,
        prev_close: 10.0 + (hash % 100.0) - 0.2,
        volume: 1000000 + (hash_i64.abs() % 10000000),
        amount: 10000000.0 + (hash % 100000000.0),
        turnover_rate: 1.0 + (hash % 10.0),
        pe: 10.0 + (hash % 50.0),
    };

    Ok(Json(quote))
}

#[derive(Debug, Serialize, Deserialize)]
struct SectorInfo {
    name: String,
    change_percent: f64,
    leader: String,
    stocks_count: i32,
}

#[derive(Deserialize)]
struct SectorQuery {
    q: Option<String>,
}

async fn get_sectors(
    Query(query): Query<SectorQuery>,
) -> AppResult<Json<Vec<SectorInfo>>> {
    let mut sectors = vec![
        SectorInfo {
            name: "软件开发".to_string(),
            change_percent: 2.35,
            leader: "中国软件".to_string(),
            stocks_count: 156,
        },
        SectorInfo {
            name: "半导体".to_string(),
            change_percent: 1.89,
            leader: "中芯国际".to_string(),
            stocks_count: 89,
        },
        SectorInfo {
            name: "新能源".to_string(),
            change_percent: -0.56,
            leader: "宁德时代".to_string(),
            stocks_count: 124,
        },
        SectorInfo {
            name: "医药".to_string(),
            change_percent: 0.78,
            leader: "恒瑞医药".to_string(),
            stocks_count: 201,
        },
        SectorInfo {
            name: "白酒".to_string(),
            change_percent: -1.23,
            leader: "贵州茅台".to_string(),
            stocks_count: 45,
        },
        SectorInfo {
            name: "人工智能".to_string(),
            change_percent: 3.45,
            leader: "科大讯飞".to_string(),
            stocks_count: 78,
        },
        SectorInfo {
            name: "新能源汽车".to_string(),
            change_percent: 1.23,
            leader: "比亚迪".to_string(),
            stocks_count: 92,
        },
        SectorInfo {
            name: "光伏".to_string(),
            change_percent: -0.89,
            leader: "隆基绿能".to_string(),
            stocks_count: 65,
        },
        SectorInfo {
            name: "芯片".to_string(),
            change_percent: 2.12,
            leader: "中芯国际".to_string(),
            stocks_count: 54,
        },
        SectorInfo {
            name: "锂电池".to_string(),
            change_percent: 0.67,
            leader: "宁德时代".to_string(),
            stocks_count: 43,
        },
    ];

    // 如果有搜索关键词，进行过滤
    if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        sectors.retain(|s| s.name.to_lowercase().contains(&q_lower));
    }

    Ok(Json(sectors))
}

#[derive(Debug, Serialize, Deserialize)]
struct HotStock {
    code: String,
    name: String,
    rank: i32,
    price: f64,
    change_percent: f64,
}

async fn get_hot_stocks() -> AppResult<Json<Vec<HotStock>>> {
    let stocks = vec![
        HotStock { code: "600519".to_string(), name: "贵州茅台".to_string(), rank: 1, price: 1688.0, change_percent: 1.25 },
        HotStock { code: "000858".to_string(), name: "五粮液".to_string(), rank: 2, price: 156.0, change_percent: 2.10 },
        HotStock { code: "300750".to_string(), name: "宁德时代".to_string(), rank: 3, price: 198.0, change_percent: -0.85 },
        HotStock { code: "601318".to_string(), name: "中国平安".to_string(), rank: 4, price: 45.0, change_percent: 0.56 },
        HotStock { code: "000001".to_string(), name: "平安银行".to_string(), rank: 5, price: 12.5, change_percent: -0.32 },
        HotStock { code: "600036".to_string(), name: "招商银行".to_string(), rank: 6, price: 32.0, change_percent: 0.89 },
        HotStock { code: "002594".to_string(), name: "比亚迪".to_string(), rank: 7, price: 256.0, change_percent: 3.21 },
        HotStock { code: "601012".to_string(), name: "隆基绿能".to_string(), rank: 8, price: 28.0, change_percent: -1.56 },
        HotStock { code: "600900".to_string(), name: "长江电力".to_string(), rank: 9, price: 25.0, change_percent: 0.45 },
        HotStock { code: "000333".to_string(), name: "美的集团".to_string(), rank: 10, price: 58.0, change_percent: 1.78 },
    ];

    Ok(Json(stocks))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/stocks/search", get(search_stocks))
        .route("/stocks/{code}/quote", get(get_stock_quote))
        .route("/sectors", get(get_sectors))
        .route("/hot-stocks", get(get_hot_stocks))
}
