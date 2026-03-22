use axum::{
    extract::{State, Path, Query},
    routing::get,
    Router,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
    Json,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::csrf;
use domain::ReplayFilter;
use domain::PaginationMeta;

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
#[template(path = "replays.html")]
struct ReplaysTemplate {
    replays: Vec<domain::ReplayWithAuthor>,
    pagination: PaginationMeta,
    search_query: String,
    search_field: String,
    sort_field: String,
    sort_order: String,
    logged_in: bool,
    username: String,
    is_admin: bool,
    tag: Option<String>,
    user_likes: std::collections::HashSet<i64>,
}

#[derive(Template)]
#[template(path = "replay_detail.html")]
struct ReplayDetailTemplate {
    replay: domain::ReplayWithAuthor,
    comments: Vec<domain::CommentWithAuthor>,
    logged_in: bool,
    username: String,
    is_admin: bool,
    current_user_id: Option<i64>,
    csrf_token: String,
    is_liked: bool,
}

#[derive(serde::Deserialize)]
struct ReplayQuery {
    page: Option<i32>,
    tag: Option<String>,
    q: Option<String>,
    search_field: Option<String>,
    sort: Option<String>,
    order: Option<String>,
}

#[derive(Deserialize)]
struct CommentForm {
    content: String,
    csrf_token: String,
}

#[derive(Deserialize)]
struct PinRequest {
    pinned: bool,
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
    let user_info = get_current_user(&cookies);
    let (logged_in, username, is_admin, user_id) = user_info
        .map(|(uid, u, a)| (true, u, a, Some(uid)))
        .unwrap_or((false, String::new(), false, None));

    let page = query.page.unwrap_or(1).max(1);
    let per_page = 20;
    let offset = (page - 1) * per_page;
    let search_query = query.q.clone().unwrap_or_default();
    let search_field = query.search_field.clone().unwrap_or_else(|| "all".to_string());
    let sort_field = query.sort.clone().unwrap_or_else(|| "created_at".to_string());
    let sort_order = query.order.clone().unwrap_or_else(|| "desc".to_string());

    let result = if let Some(ref q) = query.q {
        if !q.trim().is_empty() {
            db::replays::search_with_filter(&state.db, q, &search_field, &sort_field, &sort_order, None, per_page, offset).await
                .map_err(|e| AppError::Internal(e.to_string()))?
        } else {
            let filter = ReplayFilter {
                is_public: Some(true),
                page: query.page,
                tag: query.tag.clone(),
                sort_field: Some(sort_field.clone()),
                sort_order: Some(sort_order.clone()),
                ..Default::default()
            };
            db::replays::list(&state.db, &filter).await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
    } else {
        let filter = ReplayFilter {
            is_public: Some(true),
            page: query.page,
            tag: query.tag.clone(),
            sort_field: Some(sort_field.clone()),
            sort_order: Some(sort_order.clone()),
            ..Default::default()
        };
        db::replays::list(&state.db, &filter).await
            .map_err(|e| AppError::Internal(e.to_string()))?
    };

    // 获取用户点赞的复盘ID列表
    let user_likes = if let Some(uid) = user_id {
        let likes = db::likes::get_user_likes_for_replays(&state.db, uid, result.items.iter().map(|r| r.replay.id).collect())
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        likes.into_iter().collect()
    } else {
        std::collections::HashSet::new()
    };

    let template = ReplaysTemplate {
        replays: result.items,
        pagination: result.pagination,
        search_query,
        search_field,
        sort_field,
        sort_order,
        logged_in,
        username,
        is_admin,
        tag: query.tag,
        user_likes,
    };
    Ok(Html(template.render().unwrap()))
}

async fn replay_detail(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user_info = get_current_user(&cookies);
    let (logged_in, username, is_admin, user_id) = user_info
        .map(|(uid, u, a)| (true, u, a, Some(uid)))
        .unwrap_or((false, String::new(), false, None));

    let replay = db::replays::find_with_author(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Replay {} not found", id)))?;

    let comments = db::comments::list_by_replay(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 检查用户是否已点赞
    let is_liked = if let Some(uid) = user_id {
        db::likes::exists(&state.db, uid, id).await
            .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        false
    };

    let csrf_token = csrf::set_csrf_cookie(&cookies);
    let template = ReplayDetailTemplate { 
        replay, 
        comments, 
        logged_in, 
        username, 
        is_admin,
        current_user_id: user_id,
        csrf_token,
        is_liked,
    };
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
    
    let (user_id, _, _) = user_info.unwrap();

    // CSRF 验证
    if !csrf::verify_csrf(&cookies, &form.csrf_token) {
        return Err(AppError::BadRequest("安全验证失败，请重试".to_string()));
    }
    
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

async fn delete_replay(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    let user_info = get_current_user(&cookies)
        .ok_or_else(|| AppError::Unauthorized)?;
    
    let (user_id, _, is_admin) = user_info;

    let replay = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;

    // 检查权限：作者或管理员可以删除
    if replay.user_id != user_id && !is_admin {
        return Err(AppError::Forbidden("您没有权限删除此复盘".to_string()));
    }

    // 检查评论：如果有评论且不是管理员，则不能删除
    let comment_count = db::comments::count_by_replay(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    if comment_count > 0 && !is_admin {
        return Err(AppError::BadRequest("该复盘已有评论，无法删除".to_string()));
    }

    db::replays::delete(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Redirect::to("/replays"))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/replays", get(replays_page))
        .route("/replays/{id}", get(replay_detail))
        .route("/replays/{id}/comments", axum::routing::post(create_comment))
        .route("/replays/{id}/delete", axum::routing::get(delete_replay))
        .route("/replays/{id}/like", axum::routing::post(toggle_like))
        .route("/admin/replays/{id}/pin", axum::routing::post(pin_replay))
}

#[derive(Serialize)]
struct LikeResponse {
    liked: bool,
    like_count: i64,
}

async fn toggle_like(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<Json<LikeResponse>, AppError> {
    let user_info = get_current_user(&cookies)
        .ok_or_else(|| AppError::Unauthorized)?;
    
    let (user_id, _, _) = user_info;

    // 检查复盘是否存在
    let replay = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;

    // 检查是否已点赞
    let is_liked = db::likes::exists(&state.db, user_id, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if is_liked {
        // 取消点赞
        db::likes::delete(&state.db, user_id, id).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    } else {
        // 添加点赞
        db::likes::create(&state.db, user_id, id).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // 获取最新点赞数
    let like_count = db::likes::count_by_replay(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(LikeResponse {
        liked: !is_liked,
        like_count,
    }))
}

/// 管理员置顶/取消置顶复盘
async fn pin_replay(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<i64>,
    Json(payload): Json<PinRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_info = get_current_user(&cookies)
        .ok_or_else(|| AppError::Unauthorized)?;
    
    let (_, _, is_admin) = user_info;
    
    // 检查是否是管理员
    if !is_admin {
        return Err(AppError::Forbidden("只有管理员可以置顶复盘".to_string()));
    }

    // 检查复盘是否存在
    let _ = db::replays::find_by_id(&state.db, id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("复盘不存在: {}", id)))?;

    // 设置置顶状态
    let success = db::replays::set_pinned(&state.db, id, payload.pinned).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !success {
        return Err(AppError::Internal("更新置顶状态失败".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "is_pinned": payload.pinned,
        "message": if payload.pinned { "已置顶" } else { "已取消置顶" }
    })))
}
