use axum::{
    extract::State,
    routing::get,
    Router,
    Json,
};
use crate::state::AppState;
use crate::error::AppResult;

async fn hot_tags(State(state): State<AppState>) -> AppResult<Json<Vec<domain::HotTag>>> {
    let tags: Vec<domain::HotTag> = vec![];
    Ok(Json(tags))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tags/hot", get(hot_tags))
}
