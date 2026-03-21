use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    success: bool,
    message: String,
    count: Option<usize>,
}

pub async fn trigger_stock_sync(
    State(state): State<AppState>,
) -> AppResult<Json<SyncResponse>> {
    match state.scheduler.trigger_sync().await {
        Ok(count) => {
            Ok(Json(SyncResponse {
                success: true,
                message: "股票数据同步成功".to_string(),
                count: Some(count),
            }))
        }
        Err(e) => {
            Ok(Json(SyncResponse {
                success: false,
                message: format!("同步失败: {}", e),
                count: None,
            }))
        }
    }
}

pub async fn sync_status(
    State(state): State<AppState>,
) -> AppResult<Json<SyncResponse>> {
    Ok(Json(SyncResponse {
        success: true,
        message: format!("定时同步状态: {}", if state.scheduler.is_running() { "运行中" } else { "已停止" }),
        count: None,
    }))
}

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/sync/stocks", axum::routing::post(trigger_stock_sync))
        .route("/sync/status", axum::routing::get(sync_status))
}
