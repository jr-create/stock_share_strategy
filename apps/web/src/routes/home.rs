use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use askama::Template;
use tower_cookies::Cookies;
use crate::state::AppState;

const SESSION_COOKIE_NAME: &str = "session";

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    title: String,
    logged_in: bool,
    username: String,
}

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

async fn home_page(cookies: Cookies) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    let template = HomeTemplate {
        title: "Stock Replay - 股票复盘记录平台".to_string(),
        logged_in,
        username,
    };
    Html(template.render().unwrap())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(home_page))
}
