use axum::{
    extract::State,
    routing::get,
    Router,
    Form,
    response::{Html, IntoResponse, Redirect, Response},
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
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "my_replays.html")]
struct MyReplaysTemplate {
    replays: Vec<domain::ReplayWithAuthor>,
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "statistics.html")]
struct StatisticsTemplate {
    total_replays: i64,
    public_replays: i64,
    total_likes: i64,
    total_comments: i64,
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "new_replay.html")]
struct NewReplayTemplate {
    logged_in: bool,
    username: String,
    error: String,
}

#[derive(Template)]
#[template(path = "edit_replay.html")]
struct EditReplayTemplate {
    replay: domain::Replay,
    tags: Vec<String>,
    logged_in: bool,
    username: String,
    error: String,
}

#[derive(Deserialize)]
struct NewReplayForm {
    title: String,
    trade_date: String,
    market_sentiment: Option<String>,
    market_view: Option<String>,
    sector_focus: Option<String>,
    watchlist: Option<String>,
    plan: Option<String>,
    execution: Option<String>,
    review: Option<String>,
    is_public: Option<String>,
    tags: Option<String>,
}

enum DashboardResponse {
    Html(Html<String>),
    Redirect(Redirect),
}

impl IntoResponse for DashboardResponse {
    fn into_response(self) -> Response {
        match self {
            DashboardResponse::Html(html) => html.into_response(),
            DashboardResponse::Redirect(redirect) => redirect.into_response(),
        }
    }
}

async fn dashboard(cookies: Cookies) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    if !logged_in {
        return DashboardResponse::Redirect(Redirect::to("/login"));
    }
    
    let template = DashboardTemplate { logged_in, username };
    DashboardResponse::Html(Html(template.render().unwrap()))
}

async fn my_replays(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, username) = user_info.unwrap();
    
    let filter = ReplayFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    
    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let template = MyReplaysTemplate {
        replays,
        logged_in: true,
        username,
    };
    Ok(DashboardResponse::Html(Html(template.render().unwrap())))
}

async fn statistics(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, username) = user_info.unwrap();
    
    let filter = ReplayFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    
    let replays = db::replays::list(&state.db, &filter).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let total_replays = replays.len() as i64;
    let public_replays = replays.iter().filter(|r| r.replay.is_public).count() as i64;
    let total_likes: i64 = replays.iter().map(|r| r.like_count).sum();
    let total_comments: i64 = replays.iter().map(|r| r.comment_count).sum();
    
    let template = StatisticsTemplate {
        total_replays,
        public_replays,
        total_likes,
        total_comments,
        logged_in: true,
        username,
    };
    Ok(DashboardResponse::Html(Html(template.render().unwrap())))
}

async fn new_replay_page(cookies: Cookies) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    if !logged_in {
        return DashboardResponse::Redirect(Redirect::to("/login"));
    }
    
    let template = NewReplayTemplate { logged_in, username, error: String::new() };
    DashboardResponse::Html(Html(template.render().unwrap()))
}

async fn create_replay(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<NewReplayForm>,
) -> Result<DashboardResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, _) = user_info.unwrap();

    let trade_date = chrono::NaiveDate::parse_from_str(&form.trade_date, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("日期格式错误: {}", e)))?;

    let tags: Vec<String> = form.tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    let new_replay = domain::NewReplay {
        title: form.title,
        trade_date,
        market_sentiment: form.market_sentiment.unwrap_or_default(),
        market_view: form.market_view.unwrap_or_default(),
        sector_focus: form.sector_focus.unwrap_or_default(),
        watchlist: form.watchlist.unwrap_or_default(),
        plan: form.plan.unwrap_or_default(),
        execution: form.execution.unwrap_or_default(),
        review: form.review.unwrap_or_default(),
        is_public: form.is_public.is_some(),
        tags,
    };

    let replay = db::replays::create(&state.db, user_id, &new_replay).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(DashboardResponse::Redirect(Redirect::to(&format!("/replays/{}", replay.id))))
}

async fn edit_replay_page(
    State(state): State<AppState>,
    cookies: Cookies,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, username) = user_info.unwrap();
    
    let replay = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;
    
    if replay.user_id != user_id {
        return Err(AppError::BadRequest("无权编辑此复盘".to_string()));
    }
    
    let tags = db::replays::find_tags(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    let template = EditReplayTemplate {
        replay,
        tags,
        logged_in: true,
        username,
        error: String::new(),
    };
    Ok(DashboardResponse::Html(Html(template.render().unwrap())))
}

async fn update_replay(
    State(state): State<AppState>,
    cookies: Cookies,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Form(form): Form<NewReplayForm>,
) -> Result<DashboardResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, _) = user_info.unwrap();
    
    let existing = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;
    
    if existing.user_id != user_id {
        return Err(AppError::BadRequest("无权编辑此复盘".to_string()));
    }

    let trade_date = chrono::NaiveDate::parse_from_str(&form.trade_date, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("日期格式错误: {}", e)))?;

    let tags: Vec<String> = form.tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    let update = domain::UpdateReplay {
        title: Some(form.title),
        trade_date: Some(trade_date),
        market_sentiment: form.market_sentiment,
        market_view: form.market_view,
        sector_focus: form.sector_focus,
        watchlist: form.watchlist,
        plan: form.plan,
        execution: form.execution,
        review: form.review,
        is_public: Some(form.is_public.is_some()),
        tags: Some(tags),
    };

    db::replays::update(&state.db, id, &update).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(DashboardResponse::Redirect(Redirect::to(&format!("/replays/{}", id))))
}

async fn delete_replay(
    State(state): State<AppState>,
    cookies: Cookies,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<DashboardResponse, AppError> {
    let user_info = get_current_user(&cookies);
    
    if user_info.is_none() {
        return Ok(DashboardResponse::Redirect(Redirect::to("/login")));
    }
    
    let (user_id, _) = user_info.unwrap();
    
    let existing = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;
    
    if existing.user_id != user_id {
        return Err(AppError::BadRequest("无权删除此复盘".to_string()));
    }

    db::replays::delete(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(DashboardResponse::Redirect(Redirect::to("/dashboard/my-replays")))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/dashboard/my-replays", get(my_replays))
        .route("/dashboard/statistics", get(statistics))
        .route("/dashboard/replays/new", get(new_replay_page).post(create_replay))
        .route("/dashboard/replays/{id}/edit", get(edit_replay_page).post(update_replay))
        .route("/dashboard/replays/{id}/delete", get(delete_replay))
}
