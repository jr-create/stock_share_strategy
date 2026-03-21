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

const SESSION_COOKIE_NAME: &str = "session";

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: String,
    logged_in: bool,
    username: String,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: String,
    logged_in: bool,
    username: String,
}

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
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

async fn login_page(cookies: Cookies) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    let template = LoginTemplate { error: String::new(), logged_in, username };
    Html(template.render().unwrap())
}

async fn register_page(cookies: Cookies) -> impl IntoResponse {
    let (logged_in, username) = get_current_user(&cookies)
        .map(|(_, u)| (true, u))
        .unwrap_or((false, String::new()));
    
    let template = RegisterTemplate { error: String::new(), logged_in, username };
    Html(template.render().unwrap())
}

enum AuthResponse {
    Html(Html<String>),
    Redirect(Redirect),
}

impl IntoResponse for AuthResponse {
    fn into_response(self) -> Response {
        match self {
            AuthResponse::Html(html) => html.into_response(),
            AuthResponse::Redirect(redirect) => redirect.into_response(),
        }
    }
}

async fn login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<AuthResponse, AppError> {
    let user = db::users::find_by_email(&state.db, &form.email).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::BadRequest("邮箱或密码错误".to_string()))?;

    if !common::password::verify(&form.password, &user.password_hash) {
        let template = LoginTemplate {
            error: "邮箱或密码错误".to_string(),
            logged_in: false,
            username: String::new(),
        };
        return Ok(AuthResponse::Html(Html(template.render().unwrap())));
    }

    let session_value = format!("{}:{}", user.id, user.username);
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_value))
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::days(7))
        .build();
    cookies.add(cookie);

    Ok(AuthResponse::Redirect(Redirect::to("/")))
}

async fn register_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<RegisterForm>,
) -> Result<AuthResponse, AppError> {
    if form.password != form.confirm_password {
        let template = RegisterTemplate {
            error: "两次密码不一致".to_string(),
            logged_in: false,
            username: String::new(),
        };
        return Ok(AuthResponse::Html(Html(template.render().unwrap())));
    }

    if let Err(e) = domain::User::validate_username(&form.username) {
        let template = RegisterTemplate { error: e, logged_in: false, username: String::new() };
        return Ok(AuthResponse::Html(Html(template.render().unwrap())));
    }

    if let Err(e) = domain::User::validate_email(&form.email) {
        let template = RegisterTemplate { error: e, logged_in: false, username: String::new() };
        return Ok(AuthResponse::Html(Html(template.render().unwrap())));
    }

    if let Err(e) = domain::User::validate_password(&form.password) {
        let template = RegisterTemplate { error: e, logged_in: false, username: String::new() };
        return Ok(AuthResponse::Html(Html(template.render().unwrap())));
    }

    let password_hash = common::password::hash(&form.password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let new_user = domain::NewUser {
        username: form.username.clone(),
        email: form.email,
        password: form.password,
    };

    let user = db::users::create(&state.db, &new_user, &password_hash).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let session_value = format!("{}:{}", user.id, user.username);
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_value))
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::days(7))
        .build();
    cookies.add(cookie);

    Ok(AuthResponse::Redirect(Redirect::to("/")))
}

async fn logout_handler(cookies: Cookies) -> impl IntoResponse {
    cookies.remove(Cookie::build((SESSION_COOKIE_NAME, "")).path("/").build());
    Redirect::to("/")
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login_handler))
        .route("/register", get(register_page).post(register_handler))
        .route("/logout", get(logout_handler))
}
