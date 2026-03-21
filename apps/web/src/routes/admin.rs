use axum::{
    extract::State,
    routing::get,
    Router,
    Form,
    response::{Html, IntoResponse, Redirect, Response},
};
use askama::Template;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::csrf;

const SESSION_COOKIE_NAME: &str = "session";
const ADMIN_USERNAME: &str = "admin";
const ADMIN_PASSWORD: &str = "admin@123";

#[derive(Template)]
#[template(path = "admin_login.html")]
struct AdminLoginTemplate {
    logged_in: bool,
    username: String,
    error: String,
    form_username: String,
    csrf_token: String,
}

#[derive(Deserialize)]
struct AdminLoginForm {
    username: String,
    password: String,
    csrf_token: String,
}

async fn admin_login_page(cookies: Cookies) -> impl IntoResponse {
    let csrf_token = csrf::set_csrf_cookie(&cookies);
    let template = AdminLoginTemplate { 
        logged_in: false,
        username: String::new(),
        error: String::new(), 
        form_username: String::new(), 
        csrf_token 
    };
    Html(template.render().unwrap())
}

enum AdminResponse {
    Html(Html<String>),
    Redirect(Redirect),
}

impl IntoResponse for AdminResponse {
    fn into_response(self) -> Response {
        match self {
            AdminResponse::Html(html) => html.into_response(),
            AdminResponse::Redirect(redirect) => redirect.into_response(),
        }
    }
}

async fn admin_login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<AdminLoginForm>,
) -> Result<AdminResponse, AppError> {
    // CSRF 验证
    if !csrf::verify_csrf(&cookies, &form.csrf_token) {
        let csrf_token = csrf::set_csrf_cookie(&cookies);
        let template = AdminLoginTemplate {
            logged_in: false,
            username: String::new(),
            error: "安全验证失败，请重试".to_string(),
            form_username: form.username,
            csrf_token,
        };
        return Ok(AdminResponse::Html(Html(template.render().unwrap())));
    }

    // 验证管理员账号密码
    if form.username != ADMIN_USERNAME || form.password != ADMIN_PASSWORD {
        let csrf_token = csrf::set_csrf_cookie(&cookies);
        let template = AdminLoginTemplate {
            logged_in: false,
            username: String::new(),
            error: "管理员账号或密码错误".to_string(),
            form_username: form.username,
            csrf_token,
        };
        return Ok(AdminResponse::Html(Html(template.render().unwrap())));
    }

    // 查找或创建管理员用户
    let admin_user = match db::users::find_by_email(&state.db, "admin@stock-replay.local").await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // 创建管理员用户
            let password_hash = common::password::hash(ADMIN_PASSWORD)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            
            let new_user = domain::NewUser {
                username: ADMIN_USERNAME.to_string(),
                email: "admin@stock-replay.local".to_string(),
                password: ADMIN_PASSWORD.to_string(),
            };
            
            let mut user = db::users::create(&state.db, &new_user, &password_hash).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            
            // 设置为管理员
            sqlx::query("UPDATE users SET is_admin = TRUE WHERE id = $1")
                .bind(user.id)
                .execute(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            
            user.is_admin = true;
            user
        }
        Err(e) => return Err(AppError::Internal(e.to_string())),
    };

    // 设置管理员 session (格式: user_id:username:is_admin)
    let session_value = format!("{}:{}:{}", admin_user.id, admin_user.username, "1");
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_value))
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::days(7))
        .build();
    cookies.add(cookie);

    Ok(AdminResponse::Redirect(Redirect::to("/replays")))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/login", get(admin_login_page).post(admin_login_handler))
}
