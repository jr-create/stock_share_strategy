use axum::{
    extract::{State, Path, Query},
    routing::{get, post, put, delete},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::error::{AppError, AppResult};
use domain::{NewReplay, UpdateReplay, ReplayFilter};

#[derive(Serialize)]
struct ReplayResponse {
    id: i64,
    title: String,
    trade_date: String,
    author: String,
    like_count: i64,
    comment_count: i64,
}

#[derive(Deserialize)]
struct CreateReplayRequest {
    title: String,
    trade_date: String,
    market_sentiment: Option<String>,
    market_view: Option<String>,
    sector_focus: Option<String>,
    watchlist: Option<String>,
    plan: Option<String>,
    execution: Option<String>,
    review: Option<String>,
    is_public: Option<bool>,
    tags: Option<Vec<String>>,
    expiry_date: Option<String>,
    is_prediction: Option<bool>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<i32>,
}

async fn list_replays(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ReplayResponse>>> {
    let filter = ReplayFilter {
        is_public: Some(true),
        ..Default::default()
    };

    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let response: Vec<ReplayResponse> = replays
        .into_iter()
        .map(|r| ReplayResponse {
            id: r.replay.id,
            title: r.replay.title,
            trade_date: r.replay.trade_date.to_string(),
            author: r.author_username,
            like_count: r.like_count,
            comment_count: r.comment_count,
        })
        .collect();

    Ok(Json(response))
}

async fn get_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<domain::ReplayWithAuthor>> {
    let replay = db::replays::find_with_author(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Replay {} not found", id)))?;

    Ok(Json(replay))
}

async fn create_replay(
    State(state): State<AppState>,
    Json(req): Json<CreateReplayRequest>,
) -> AppResult<Json<domain::Replay>> {
    let trade_date = chrono::NaiveDate::parse_from_str(&req.trade_date, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("Invalid date format: {}", e)))?;
    
    let expiry_date = req.expiry_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let new_replay = NewReplay {
        title: req.title,
        trade_date,
        market_sentiment: req.market_sentiment.unwrap_or_default(),
        market_view: req.market_view.unwrap_or_default(),
        sector_focus: req.sector_focus.unwrap_or_default(),
        watchlist: req.watchlist.unwrap_or_default(),
        plan: req.plan.unwrap_or_default(),
        execution: req.execution.unwrap_or_default(),
        review: req.review.unwrap_or_default(),
        is_public: req.is_public.unwrap_or(true),
        expiry_date,
        is_prediction: req.is_prediction.unwrap_or(false),
        tags: req.tags.unwrap_or_default(),
    };

    let user_id = 1i64;
    let replay = db::replays::create(&state.db, user_id, &new_replay).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(replay))
}

async fn update_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<CreateReplayRequest>,
) -> AppResult<Json<domain::Replay>> {
    let trade_date = req.trade_date.parse::<chrono::NaiveDate>().ok();
    let expiry_date = req.expiry_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let update = UpdateReplay {
        title: Some(req.title),
        trade_date,
        market_sentiment: req.market_sentiment,
        market_view: req.market_view,
        sector_focus: req.sector_focus,
        watchlist: req.watchlist,
        plan: req.plan,
        execution: req.execution,
        review: req.review,
        is_public: req.is_public,
        expiry_date,
        is_prediction: req.is_prediction,
        tags: req.tags,
    };

    let replay = db::replays::update(&state.db, id, &update).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Replay {} not found", id)))?;

    Ok(Json(replay))
}

async fn delete_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let deleted = db::replays::delete(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if deleted {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(AppError::NotFound(format!("Replay {} not found", id)))
    }
}

async fn like_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = 1i64;
    
    db::likes::create(&state.db, user_id, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn search_replays(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<ReplayResponse>>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = 20;
    let offset = (page - 1) * per_page;

    let replays = db::replays::search(&state.db, &query.q, None, per_page, offset).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let response: Vec<ReplayResponse> = replays
        .into_iter()
        .map(|r| ReplayResponse {
            id: r.replay.id,
            title: r.replay.title,
            trade_date: r.replay.trade_date.to_string(),
            author: r.author_username,
            like_count: r.like_count,
            comment_count: r.comment_count,
        })
        .collect();

    Ok(Json(response))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/replays", get(list_replays).post(create_replay))
        .route("/replays/search", get(search_replays))
        .route("/replays/{id}", get(get_replay).put(update_replay).delete(delete_replay))
        .route("/replays/{id}/like", post(like_replay))
}
