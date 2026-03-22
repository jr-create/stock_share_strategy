use axum::{
    extract::{State, Path},
    routing::{get, post},
    Router,
    response::{IntoResponse, Redirect, Response, Json},
};
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;
use domain::ReplayFilter;
use serde::{Serialize, Deserialize};

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

fn require_admin(cookies: &Cookies) -> Result<(i64, String), AppError> {
    let user_info = get_current_user(cookies)
        .ok_or(AppError::Unauthorized)?;
    
    let (user_id, username, is_admin) = user_info;
    
    if !is_admin {
        return Err(AppError::Forbidden("需要管理员权限".to_string()));
    }
    
    Ok((user_id, username))
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
        // 管理员路由
        .route("/admin/export/json", get(admin_export_json))
        .route("/admin/export/csv", get(admin_export_csv))
        .route("/admin/replays/batch-delete", post(admin_batch_delete))
}

// 管理员批量删除请求
#[derive(Deserialize)]
struct BatchDeleteRequest {
    ids: Vec<i64>,
}

// 管理员批量删除响应
#[derive(Serialize)]
struct BatchDeleteResponse {
    success: bool,
    deleted_count: usize,
    message: String,
}

/// 管理员导出所有公开复盘为JSON
async fn admin_export_json(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    require_admin(&cookies)?;
    
    let filter = ReplayFilter {
        is_public: Some(true),
        ..Default::default()
    };
    
    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let export_data: Vec<ReplayExport> = replays.items.into_iter().map(ReplayExport::from).collect();
    
    let json_content = serde_json::to_string_pretty(&export_data)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let filename = format!("all_public_replays_{}.json", timestamp);
    
    Ok((
        [
            ("Content-Type", "application/json"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        json_content,
    ).into_response())
}

/// 管理员导出所有公开复盘为CSV
async fn admin_export_csv(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    require_admin(&cookies)?;
    
    let filter = ReplayFilter {
        is_public: Some(true),
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
    let filename = format!("all_public_replays_{}.csv", timestamp);
    
    Ok((
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
        ],
        csv_content,
    ).into_response())
}

/// 管理员批量删除复盘
async fn admin_batch_delete(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<BatchDeleteRequest>,
) -> Result<Json<BatchDeleteResponse>, AppError> {
    require_admin(&cookies)?;
    
    if request.ids.is_empty() {
        return Ok(Json(BatchDeleteResponse {
            success: false,
            deleted_count: 0,
            message: "未选择要删除的复盘".to_string(),
        }));
    }
    
    let mut deleted_count = 0;
    for id in &request.ids {
        match db::replays::delete(&state.db, *id).await {
            Ok(true) => deleted_count += 1,
            Ok(false) => {},
            Err(_) => {},
        }
    }
    
    Ok(Json(BatchDeleteResponse {
        success: true,
        deleted_count,
        message: format!("成功删除 {} 条复盘记录", deleted_count),
    }))
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
