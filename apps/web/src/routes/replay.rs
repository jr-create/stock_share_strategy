use axum::{
    extract::{State, Path, Query},
    routing::get,
    Router,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use askama::Template;
use serde::Deserialize;
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;
use domain::ReplayFilter;

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
#[template(path = "replays.html")]
struct ReplaysTemplate {
    replays: Vec<domain::ReplayWithAuthor>,
    page: i32,
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "replay_detail.html")]
struct ReplayDetailTemplate {
    replay: domain::ReplayWithAuthor,
    comments: Vec<domain::CommentWithAuthor>,
    logged_in: bool,
    username: String,
}

#[derive(serde::Deserialize)]
struct ReplayQuery {
    page: Option<i32>,
    tag: Option<String>,
}

#[derive(Deserialize)]
struct CommentForm {
    content: String,
}

enum ReplayResponse {
    Html(Html<String>),
    Redirect(Redirect),
}

impl IntoResponse for ReplayResponse {
    fn into_response(self) -> Response {
        match self {
            ReplayResponse::Html(html) => html.into_response(),
            ReplayResponse::Redirect(redirect) => redirect.into_response(),
        }
    }
}

async fn replays_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<ReplayQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));

    let filter = ReplayFilter {
        is_public: Some(true),
        page: query.page,
        tag: query.tag,
        ..Default::default()
    };

    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let template = ReplaysTemplate {
        replays,
        page: filter.page(),
        logged_in,
        username,
    };
    Ok(Html(template.render().unwrap()))
}

async fn replay_detail(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));

    let replay = db::replays::find_with_author(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Replay {} not found", id)))?;

    let comments = db::comments::list_by_replay(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let template = ReplayDetailTemplate { replay, comments, logged_in, username };
    Ok(Html(template.render().unwrap()))
}

async fn create_comment(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
    Form(form): Form<CommentForm>,
) -> Result<ReplayResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(ReplayResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, _) = user_info.unwrap();
    
    let content = form.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("评论内容不能为空".to_string()));
    }
    
    let new_comment = domain::NewComment {
        replay_id: id,
        content: content.to_string(),
    };
    
    db::comments::create(&state.db, user_id, &new_comment).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(ReplayResponse::Redirect(Redirect::to(&format!("/replays/{}#comments", id))))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/replays", get(replays_page))
        .route("/replays/{id}", get(replay_detail))
        .route("/replays/{id}/comments", axum::routing::post(create_comment))
}
