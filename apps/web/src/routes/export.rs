use axum::{
    extract::{State, Path},
    routing::get,
    Router,
    response::{IntoResponse, Redirect, Response},
};
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;
use domain::ReplayFilter;
use serde::Serialize;

const SESSION_COOKIE_NAME: &str = "session";

fn get_current_user(cookies: &Cookies) -> Option<(i64, String, bool)> {
    cookies.get(SESSION_COOKIE_NAME).and_then(|c| {
        let value = c.value();
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() >= 2 {
            let is_admin = parts.get(2).map(|&s| s == "1").unwrap_or(false);
            Some((parts[0].parse().ok()?, parts[1].to_string(), is_admin))
        } else {
            None
        }
    })
}

#[derive(Serialize)]
struct ReplayExport {
    title: String,
    trade_date: String,
    market_sentiment: String,
    market_view: String,
    sector_focus: String,
    watchlist: String,
    plan: String,
    execution: String,
    review: String,
    is_public: bool,
    is_prediction: bool,
    expiry_date: Option<String>,
    tags: String,
    created_at: String,
    updated_at: String,
}

impl From<domain::ReplayWithAuthor> for ReplayExport {
    fn from(r: domain::ReplayWithAuthor) -> Self {
        ReplayExport {
            title: r.replay.title,
            trade_date: r.replay.trade_date.to_string(),
            market_sentiment: r.replay.market_sentiment,
            market_view: r.replay.market_view,
            sector_focus: r.replay.sector_focus,
            watchlist: r.replay.watchlist,
            plan: r.replay.plan,
            execution: r.replay.execution,
            review: r.replay.review,
            is_public: r.replay.is_public,
            is_prediction: r.replay.is_prediction,
            expiry_date: r.replay.expiry_date.map(|d| d.to_string()),
            tags: r.tags.join(", "),
            created_at: r.replay.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: r.replay.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

async fn export_json(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    
    let (user_id, _, _) = user_info.unwrap();
    
    let filter = ReplayFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    
    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let export_data: Vec<ReplayExport> = replays.items.into_iter().map(ReplayExport::from).collect();
    
    let json_content = serde_json::to_string_pretty(&export_data)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let filename = format!("replays_{}.json", timestamp);
    
    Ok((
        [
            ("Content-Type", "application/json"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        json_content,
    ).into_response())
}

async fn export_csv(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    
    let (user_id, _, _) = user_info.unwrap();
    
    let filter = ReplayFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    
    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let export_data: Vec<ReplayExport> = replays.items.into_iter().map(ReplayExport::from).collect();
    
    let mut csv_content = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut csv_content);
        
        for record in export_data {
            wtr.serialize(record)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        
        wtr.flush().map_err(|e: std::io::Error| AppError::Internal(e.to_string()))?;
    }
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let filename = format!("replays_{}.csv", timestamp);
    
    Ok((
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        csv_content,
    ).into_response())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/my/export/json", get(export_json))
        .route("/my/export/csv", get(export_csv))
        .route("/my/export/{id}/json", get(export_single_json))
        .route("/my/export/{id}/csv", get(export_single_csv))
}

async fn export_single_json(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    
    let (user_id, _, _) = user_info.unwrap();
    
    let replay = db::replays::find_with_author(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;
    
    if replay.replay.user_id != user_id {
        return Err(AppError::Forbidden("您没有权限导出此复盘".to_string()));
    }
    
    let export_data = ReplayExport::from(replay);
    
    let json_content = serde_json::to_string_pretty(&export_data)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let filename = format!("replay_{}.json", id);
    
    Ok((
        [
            ("Content-Type", "application/json"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        json_content,
    ).into_response())
}

async fn export_single_csv(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    
    let (user_id, _, _) = user_info.unwrap();
    
    let replay = db::replays::find_with_author(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;
    
    if replay.replay.user_id != user_id {
        return Err(AppError::Forbidden("您没有权限导出此复盘".to_string()));
    }
    
    let export_data = ReplayExport::from(replay);
    
    let mut csv_content = Vec::new();
    {
        let mut wtr = csv::Writer::from_writer(&mut csv_content);
        wtr.serialize(&export_data)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        wtr.flush().map_err(|e: std::io::Error| AppError::Internal(e.to_string()))?;
    }
    
    let filename = format!("replay_{}.csv", id);
    
    Ok((
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        csv_content,
    ).into_response())
}
