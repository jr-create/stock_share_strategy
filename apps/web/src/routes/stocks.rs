use axum::{
    extract::{State, Path},
    routing::get,
    Router,
    response::{Html, IntoResponse},
};
use askama::Template;
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;

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

#[derive(Template)]
#[template(path = "stock_detail.html")]
struct StockDetailTemplate {
    stock: domain::Stock,
    logged_in: bool,
    username: String,
}

async fn stock_detail(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u, _)| (true, u))
        .unwrap_or((false, String::new()));

    let stock = db::stocks::find_by_code(&state.db, &code).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Stock {} not found", code)))?;

    let template = StockDetailTemplate { stock, logged_in, username };
    Ok(Html(template.render().unwrap()))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/stocks/{code}", get(stock_detail))
}
