use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse},
};
use askama::Template;
use tower_cookies::Cookies;
use crate::state::AppState;

const SESSION_COOKIE_NAME: &str = "session";

fn get_current_user(cookies: &Cookies) -> Option<(i64, String)> {
    cookies.get(SESSION_COOKIE_NAME).and_then(|c| {
        let value = c.value();
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() == 2 {
            Some((parts[0].parse().ok()?, parts[1].to_string()))
        } else {
            None
        }
    })
}

#[derive(Template)]
#[template(path = "market.html")]
struct MarketTemplate {
    logged_in: bool,
    username: String,
}

async fn market_page(
    _state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    let template = MarketTemplate { logged_in, username };
    Html(template.render().unwrap())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/market", get(market_page))
}
