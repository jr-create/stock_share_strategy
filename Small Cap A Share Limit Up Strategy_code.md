Small Cap A Share Limit Up Strategy


**skai**: 可以，我继续。
先说明一个边界：你知识库里**只有这份项目方案**，明确给出了：

- 技术栈：**Axum + Askama + SQLx + PostgreSQL + Tailwind + Docker Compose**
- 目录结构建议
- 页面路由与 JSON API
- 数据库表结构
- `Cargo.toml` 与一个**不完整**的 `main.rs` 片段[1]

所以你下面看到的内容里：

- **已明确存在于笔记中的部分**：我会直接沿用[1]
- **笔记里没给出具体实现的部分**：我会基于同一份方案，补成一套**一致的 MVP scaffold**，方便你直接继续开发[1]

---
# Repository dump
## `/Cargo.toml`

```toml
[workspace]
members = [
    "apps/web",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"

[workspace.dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "trace", "compression-br"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
askama = "0.12"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "macros", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
anyhow = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }

```

---
## `/apps/web/Cargo.toml`

```toml
[package]
name = "stock-replay-web"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { workspace = true }
tokio = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
askama = { workspace = true }
sqlx = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }

```

---
## `/apps/web/src/main.rs`

> 这里沿用了你笔记中已有的 `axum` / `ServeDir` / `TraceLayer` / tracing 初始化方向，并把被截断的 `main.rs` 补完整为可运行 MVP。[1]
```rust
mod app;
mod config;
mod error;
mod handlers;
mod models;
mod repos;
mod routes;
mod services;
mod state;
mod views;

use std::net::SocketAddr;

use anyhow::Context;
use tokio::net::TcpListener;
use tracing::info;

use crate::app::build_app;
use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=info")
        .init();

    let config = Config::from_env()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .context("failed to connect to postgres")?;

    let state = AppState::new(config.clone(), pool);
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .context("failed to bind tcp listener")?;

    info!("stock-replay-web listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}

```

---
## `/apps/web/src/app.rs`

```rust
use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    services::ServeDir,
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .merge(crate::routes::pages::router())
        .merge(crate::routes::api::router())
        .nest_service("/static", ServeDir::new("apps/web/static"))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

```

---
## `/apps/web/src/config.rs`

```rust
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("APP_PORT must be a valid u16")?;

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/stock_replay".to_string());

        let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a valid u32")?;

        Ok(Self {
            port,
            database_url,
            database_max_connections,
        })
    }
}

```

---
## `/apps/web/src/state.rs`

```rust
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self { config, db }
    }
}

```

---
## `/apps/web/src/error.rs`

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Db(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database error".to_string(),
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not found".to_string(),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                msg,
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
        };

        let body = Json(ErrorBody { error: message });
        (status, body).into_response()
    }
}

```

---
## `/migrations/0001_init.sql`

> 这里直接按你笔记里的表结构落成初始化 SQL：`users`、`replays`、`replay_tags`、`comments`、`likes`、`stocks`。[1]
```sql
create table if not exists users (
  id bigserial primary key,
  username varchar(32) unique not null,
  email varchar(128) unique not null,
  password_hash text not null,
  bio text not null default '',
  avatar_url text not null default '',
  created_at timestamptz not null default now()
);

create table if not exists replays (
  id bigserial primary key,
  user_id bigint not null references users(id) on delete cascade,
  title varchar(200) not null,
  trade_date date not null,
  market_sentiment varchar(32) not null default '',
  market_view text not null default '',
  sector_focus text not null default '',
  watchlist text not null default '',
  plan text not null default '',
  execution text not null default '',
  review text not null default '',
  is_public boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists replay_tags (
  replay_id bigint not null references replays(id) on delete cascade,
  tag varchar(50) not null,
  primary key (replay_id, tag)
);

create table if not exists comments (
  id bigserial primary key,
  replay_id bigint not null references replays(id) on delete cascade,
  user_id bigint not null references users(id) on delete cascade,
  content text not null,
  created_at timestamptz not null default now()
);

create table if not exists likes (
  replay_id bigint not null references replays(id) on delete cascade,
  user_id bigint not null references users(id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (replay_id, user_id)
);

create table if not exists stocks (
  code varchar(16) primary key,
  name varchar(64) not null,
  market varchar(16) not null default '',
  sector varchar(64) not null default '',
  updated_at timestamptz not null default now()
);

```

---
## `/migrations/0002_indexes.sql`

> 这部分笔记没有给出现成索引，我按你方案里的核心查询路径补了 MVP 索引：复盘时间排序、公开广场、评论、标签、热标签、个股搜索。[1]
```sql
create index if not exists idx_replays_user_id on replays(user_id);
create index if not exists idx_replays_trade_date on replays(trade_date desc);
create index if not exists idx_replays_created_at on replays(created_at desc);
create index if not exists idx_replays_public_trade_date on replays(is_public, trade_date desc);

create index if not exists idx_comments_replay_id on comments(replay_id);
create index if not exists idx_comments_user_id on comments(user_id);
create index if not exists idx_comments_created_at on comments(created_at desc);

create index if not exists idx_replay_tags_tag on replay_tags(tag);
create index if not exists idx_stocks_name on stocks(name);
create index if not exists idx_stocks_sector on stocks(sector);

```

---
## `/docker-compose.yml`

> 你的笔记明确把 Docker Compose 作为开发环境的一部分，但没给文件内容；这里我补成最小可用的 Postgres 开发栈。[1]
```yaml
version: "3.9"

services:
  postgres:
    image: postgres:16
    container_name: stock-replay-postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: stock_replay
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    ports:
      - "5432:5432"
    volumes:
      - stock-replay-postgres-data:/var/lib/postgresql/data

volumes:
  stock-replay-postgres-data:

```

---
# Remaining routes / handlers / templates MVP scaffold
## `/apps/web/src/routes/mod.rs`

```rust
pub mod api;
pub mod pages;

```

---
## `/apps/web/src/routes/pages.rs`

> 页面路由是直接按你笔记中的页面设计来接：[1]
```rust
use axum::{routing::get, Router};

use crate::{
    handlers::pages,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(pages::home))
        .route("/replays", get(pages::replays_index))
        .route("/replays/:id", get(pages::replay_detail))
        .route("/stocks/:code", get(pages::stock_detail))
        .route("/tags/:tag", get(pages::tag_detail))
        .route("/u/:username", get(pages::user_profile))
        .route("/login", get(pages::login))
        .route("/register", get(pages::register))
        .route("/dashboard", get(pages::dashboard))
        .route("/dashboard/replays/new", get(pages::dashboard_new_replay))
}

```

---
## `/apps/web/src/routes/api.rs`

> API 也是按你笔记里的清单来接：[1]
```rust
use axum::{routing::{delete, get, post, put}, Router};

use crate::{
    handlers::api,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/replays", get(api::list_replays).post(api::create_replay))
        .route(
            "/api/replays/:id",
            get(api::get_replay).put(api::update_replay).delete(api::delete_replay),
        )
        .route("/api/replays/:id/like", post(api::like_replay))
        .route("/api/replays/:id/comments", post(api::create_comment))
        .route("/api/stocks/search", get(api::search_stocks))
        .route("/api/tags/hot", get(api::hot_tags))
}

```

---
## `/apps/web/src/handlers/mod.rs`

```rust
pub mod api;
pub mod pages;

```

---
## `/apps/web/src/models/mod.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Replay {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub trade_date: NaiveDate,
    pub market_sentiment: String,
    pub market_view: String,
    pub sector_focus: String,
    pub watchlist: String,
    pub plan: String,
    pub execution: String,
    pub review: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayListItem {
    pub id: i64,
    pub title: String,
    pub trade_date: NaiveDate,
    pub market_sentiment: String,
    pub author_name: String,
    pub like_count: i64,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReplayInput {
    pub user_id: i64,
    pub title: String,
    pub trade_date: NaiveDate,
    pub market_sentiment: Option<String>,
    pub market_view: Option<String>,
    pub sector_focus: Option<String>,
    pub watchlist: Option<String>,
    pub plan: Option<String>,
    pub execution: Option<String>,
    pub review: Option<String>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateReplayInput {
    pub title: Option<String>,
    pub trade_date: Option<NaiveDate>,
    pub market_sentiment: Option<String>,
    pub market_view: Option<String>,
    pub sector_focus: Option<String>,
    pub watchlist: Option<String>,
    pub plan: Option<String>,
    pub execution: Option<String>,
    pub review: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub bio: String,
    pub avatar_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: i64,
    pub replay_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommentInput {
    pub user_id: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub market: String,
    pub sector: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotTag {
    pub tag: String,
    pub count: i64,
}

```

---
## `/apps/web/src/repos/mod.rs`

```rust
pub mod replay_repo;
pub mod stock_repo;
pub mod tag_repo;

```

---
## `/apps/web/src/repos/replay_repo.rs`

```rust
use sqlx::PgPool;

use crate::models::{Comment, CreateCommentInput, CreateReplayInput, Replay, ReplayListItem, UpdateReplayInput};

pub async fn list_public_replays(pool: &PgPool) -> Result<Vec<ReplayListItem>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ReplayListItem,
        r#"
        select
            r.id,
            r.title,
            r.trade_date,
            r.market_sentiment,
            u.username as author_name,
            coalesce((select count(*) from likes l where l.replay_id = r.id), 0)::bigint as like_count,
            coalesce((select count(*) from comments c where c.replay_id = r.id), 0)::bigint as comment_count
        from replays r
        join users u on u.id = r.user_id
        where r.is_public = true
        order by r.trade_date desc, r.id desc
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_replay_by_id(pool: &PgPool, id: i64) -> Result<Option<Replay>, sqlx::Error> {
    let replay = sqlx::query_as::<_, Replay>(
        r#"
        select *
        from replays
        where id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(replay)
}

pub async fn create_replay(pool: &PgPool, input: &CreateReplayInput) -> Result<Replay, sqlx::Error> {
    let replay = sqlx::query_as::<_, Replay>(
        r#"
        insert into replays (
            user_id,
            title,
            trade_date,
            market_sentiment,
            market_view,
            sector_focus,
            watchlist,
            plan,
            execution,
            review,
            is_public
        )
        values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        returning *
        "#,
    )
    .bind(input.user_id)
    .bind(&input.title)
    .bind(input.trade_date)
    .bind(input.market_sentiment.clone().unwrap_or_default())
    .bind(input.market_view.clone().unwrap_or_default())
    .bind(input.sector_focus.clone().unwrap_or_default())
    .bind(input.watchlist.clone().unwrap_or_default())
    .bind(input.plan.clone().unwrap_or_default())
    .bind(input.execution.clone().unwrap_or_default())
    .bind(input.review.clone().unwrap_or_default())
    .bind(input.is_public.unwrap_or(true))
    .fetch_one(pool)
    .await?;

    if let Some(tags) = &input.tags {
        for tag in tags {
            sqlx::query(
                r#"
                insert into replay_tags (replay_id, tag)
                values ($1, $2)
                on conflict do nothing
                "#,
            )
            .bind(replay.id)
            .bind(tag)
            .execute(pool)
            .await?;
        }
    }

    Ok(replay)
}

pub async fn update_replay(
    pool: &PgPool,
    id: i64,
    input: &UpdateReplayInput,
) -> Result<Option<Replay>, sqlx::Error> {
    let existing = get_replay_by_id(pool, id).await?;
    let Some(existing) = existing else {
        return Ok(None);
    };

    let replay = sqlx::query_as::<_, Replay>(
        r#"
        update replays
        set
            title = $2,
            trade_date = $3,
            market_sentiment = $4,
            market_view = $5,
            sector_focus = $6,
            watchlist = $7,
            plan = $8,
            execution = $9,
            review = $10,
            is_public = $11,
            updated_at = now()
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(input.title.clone().unwrap_or(existing.title))
    .bind(input.trade_date.unwrap_or(existing.trade_date))
    .bind(input.market_sentiment.clone().unwrap_or(existing.market_sentiment))
    .bind(input.market_view.clone().unwrap_or(existing.market_view))
    .bind(input.sector_focus.clone().unwrap_or(existing.sector_focus))
    .bind(input.watchlist.clone().unwrap_or(existing.watchlist))
    .bind(input.plan.clone().unwrap_or(existing.plan))
    .bind(input.execution.clone().unwrap_or(existing.execution))
    .bind(input.review.clone().unwrap_or(existing.review))
    .bind(input.is_public.unwrap_or(existing.is_public))
    .fetch_one(pool)
    .await?;

    Ok(Some(replay))
}

pub async fn delete_replay(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        delete from replays
        where id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn like_replay(pool: &PgPool, replay_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        insert into likes (replay_id, user_id)
        values ($1, $2)
        on conflict do nothing
        "#,
    )
    .bind(replay_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_comment(
    pool: &PgPool,
    replay_id: i64,
    input: &CreateCommentInput,
) -> Result<Comment, sqlx::Error> {
    let comment = sqlx::query_as::<_, Comment>(
        r#"
        insert into comments (replay_id, user_id, content)
        values ($1, $2, $3)
        returning *
        "#,
    )
    .bind(replay_id)
    .bind(input.user_id)
    .bind(&input.content)
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

```

---
## `/apps/web/src/repos/stock_repo.rs`

```rust
use sqlx::PgPool;

use crate::models::Stock;

pub async fn search_stocks(pool: &PgPool, q: &str) -> Result<Vec<Stock>, sqlx::Error> {
    let pattern = format!("%{}%", q);

    let rows = sqlx::query_as::<_, Stock>(
        r#"
        select *
        from stocks
        where code ilike $1
           or name ilike $1
        order by code asc
        limit 20
        "#,
    )
    .bind(pattern)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_stock_by_code(pool: &PgPool, code: &str) -> Result<Option<Stock>, sqlx::Error> {
    let stock = sqlx::query_as::<_, Stock>(
        r#"
        select *
        from stocks
        where code = $1
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    Ok(stock)
}

```

---
## `/apps/web/src/repos/tag_repo.rs`

```rust
use sqlx::PgPool;

use crate::models::HotTag;

pub async fn hot_tags(pool: &PgPool) -> Result<Vec<HotTag>, sqlx::Error> {
    let rows = sqlx::query_as!(
        HotTag,
        r#"
        select
            tag,
            count(*)::bigint as count
        from replay_tags
        group by tag
        order by count desc, tag asc
        limit 20
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

```

---
## `/apps/web/src/services/mod.rs`

```rust
pub mod replay_service;

```

---
## `/apps/web/src/services/replay_service.rs`

```rust
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{CreateCommentInput, CreateReplayInput, HotTag, Replay, ReplayListItem, Stock, UpdateReplayInput},
    repos::{replay_repo, stock_repo, tag_repo},
};

pub async fn list_replays(pool: &PgPool) -> Result<Vec<ReplayListItem>, AppError> {
    Ok(replay_repo::list_public_replays(pool).await?)
}

pub async fn get_replay(pool: &PgPool, id: i64) -> Result<Replay, AppError> {
    replay_repo::get_replay_by_id(pool, id)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_replay(pool: &PgPool, input: &CreateReplayInput) -> Result<Replay, AppError> {
    Ok(replay_repo::create_replay(pool, input).await?)
}

pub async fn update_replay(pool: &PgPool, id: i64, input: &UpdateReplayInput) -> Result<Replay, AppError> {
    replay_repo::update_replay(pool, id, input)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn delete_replay(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let deleted = replay_repo::delete_replay(pool, id).await?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn like_replay(pool: &PgPool, replay_id: i64, user_id: i64) -> Result<(), AppError> {
    replay_repo::like_replay(pool, replay_id, user_id).await?;
    Ok(())
}

pub async fn create_comment(
    pool: &PgPool,
    replay_id: i64,
    input: &CreateCommentInput,
) -> Result<crate::models::Comment, AppError> {
    Ok(replay_repo::create_comment(pool, replay_id, input).await?)
}

pub async fn search_stocks(pool: &PgPool, q: &str) -> Result<Vec<Stock>, AppError> {
    Ok(stock_repo::search_stocks(pool, q).await?)
}

pub async fn hot_tags(pool: &PgPool) -> Result<Vec<HotTag>, AppError> {
    Ok(tag_repo::hot_tags(pool).await?)
}

```

---
## `/apps/web/src/handlers/api.rs`

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::AppError,
    models::{CreateCommentInput, CreateReplayInput, UpdateReplayInput},
    services::replay_service,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct StockSearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct LikeReplayInput {
    pub user_id: i64,
}

pub async fn list_replays(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = replay_service::list_replays(&state.db).await?;
    Ok(Json(json!({ "items": rows })))
}

pub async fn create_replay(
    State(state): State<AppState>,
    Json(input): Json<CreateReplayInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let replay = replay_service::create_replay(&state.db, &input).await?;
    Ok((StatusCode::CREATED, Json(json!({ "item": replay }))))
}

pub async fn get_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let replay = replay_service::get_replay(&state.db, id).await?;
    Ok(Json(json!({ "item": replay })))
}

pub async fn update_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateReplayInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let replay = replay_service::update_replay(&state.db, id, &input).await?;
    Ok(Json(json!({ "item": replay })))
}

pub async fn delete_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    replay_service::delete_replay(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn like_replay(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<LikeReplayInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    replay_service::like_replay(&state.db, id, input.user_id).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn create_comment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let comment = replay_service::create_comment(&state.db, id, &input).await?;
    Ok((StatusCode::CREATED, Json(json!({ "item": comment }))))
}

pub async fn search_stocks(
    State(state): State<AppState>,
    Query(query): Query<StockSearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = replay_service::search_stocks(&state.db, &query.q).await?;
    Ok(Json(json!({ "items": rows })))
}

pub async fn hot_tags(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = replay_service::hot_tags(&state.db).await?;
    Ok(Json(json!({ "items": rows })))
}

```

---
## `/apps/web/src/views/mod.rs`

```rust
pub mod templates;

pub use templates::{HomeTemplate, ReplayDetailTemplate, ReplayListTemplate, SimplePageTemplate};

```

---
## `/apps/web/src/views/templates.rs`

```rust
use askama::Template;

use crate::models::{Replay, ReplayListItem};

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub replays: Vec<ReplayListItem>,
}

#[derive(Template)]
#[template(path = "replays/index.html")]
pub struct ReplayListTemplate {
    pub replays: Vec<ReplayListItem>,
}

#[derive(Template)]
#[template(path = "replays/detail.html")]
pub struct ReplayDetailTemplate {
    pub replay: Replay,
}

#[derive(Template)]
#[template(path = "simple_page.html")]
pub struct SimplePageTemplate<'a> {
    pub title: &'a str,
    pub heading: &'a str,
    pub body: &'a str,
}

```

---
## `/apps/web/src/handlers/pages.rs`

```rust
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{
    error::AppError,
    repos::{replay_repo, stock_repo},
    state::AppState,
    views::{HomeTemplate, ReplayDetailTemplate, ReplayListTemplate, SimplePageTemplate},
};

pub async fn home(
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let replays = replay_repo::list_public_replays(&state.db).await?;
    let tpl = HomeTemplate { replays };
    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn replays_index(
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let replays = replay_repo::list_public_replays(&state.db).await?;
    let tpl = ReplayListTemplate { replays };
    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn replay_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let replay = replay_repo::get_replay_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let tpl = ReplayDetailTemplate { replay };
    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn stock_detail(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Html<String>, AppError> {
    let stock = stock_repo::get_stock_by_code(&state.db, &code).await?;
    let body = match stock {
        Some(stock) => format!("{} ({}) · {} · {}", stock.name, stock.code, stock.market, stock.sector),
        None => format!("Stock {} not found", code),
    };

    let tpl = SimplePageTemplate {
        title: "Stock Detail",
        heading: "个股详情",
        body: &body,
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn tag_detail(
    Path(tag): Path<String>,
) -> Result<Html<String>, AppError> {
    let body = format!("Tag page for: {}", tag);

    let tpl = SimplePageTemplate {
        title: "Tag Detail",
        heading: "标签页",
        body: &body,
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn user_profile(
    Path(username): Path<String>,
) -> Result<Html<String>, AppError> {
    let body = format!("User profile for: {}", username);

    let tpl = SimplePageTemplate {
        title: "User Profile",
        heading: "用户主页",
        body: &body,
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn login() -> Result<Html<String>, AppError> {
    let tpl = SimplePageTemplate {
        title: "Login",
        heading: "登录页",
        body: "MVP scaffold: login form goes here.",
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn register() -> Result<Html<String>, AppError> {
    let tpl = SimplePageTemplate {
        title: "Register",
        heading: "注册页",
        body: "MVP scaffold: register form goes here.",
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn dashboard() -> Result<Html<String>, AppError> {
    let tpl = SimplePageTemplate {
        title: "Dashboard",
        heading: "用户控制台",
        body: "MVP scaffold: dashboard overview goes here.",
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

pub async fn dashboard_new_replay() -> Result<Html<String>, AppError> {
    let tpl = SimplePageTemplate {
        title: "New Replay",
        heading: "新建复盘",
        body: "MVP scaffold: new replay form goes here.",
    };

    Ok(Html(tpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

```

---
# Templates
## `/apps/web/templates/base.html`

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{% block title %}Stock Replay{% endblock %}</title>
  <style>
    :root {
      --bg: #0b1020;
      --panel: #121933;
      --card: #18213f;
      --text: #e8ecf8;
      --muted: #9fb0d1;
      --line: #2a365f;
      --accent: #5ea1ff;
      --green: #30c48d;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
    }
    a { color: var(--accent); text-decoration: none; }
    .container {
      max-width: 1080px;
      margin: 0 auto;
      padding: 24px;
    }
    .nav {
      display: flex;
      gap: 16px;
      padding: 16px 0 24px;
      border-bottom: 1px solid var(--line);
      margin-bottom: 24px;
    }
    .hero, .card {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 16px;
      padding: 20px;
      margin-bottom: 20px;
    }
    .list {
      display: grid;
      gap: 16px;
    }
    .muted { color: var(--muted); }
    .pill {
      display: inline-block;
      padding: 4px 10px;
      border-radius: 999px;
      background: rgba(94, 161, 255, 0.12);
      color: var(--accent);
      font-size: 12px;
    }
    h1, h2, h3 { margin-top: 0; }
    pre {
      white-space: pre-wrap;
      word-break: break-word;
    }
  </style>
</head>
<body>
  <div class="container">
    <nav class="nav">
      <a href="/">首页</a>
      <a href="/replays">复盘广场</a>
      <a href="/dashboard">控制台</a>
      <a href="/login">登录</a>
      <a href="/register">注册</a>
    </nav>

    {% block content %}{% endblock %}
  </div>
</body>
</html>

```

---
## `/apps/web/templates/home.html`

```html
{% extends "base.html" %}

{% block title %}Stock Replay - 首页{% endblock %}

{% block content %}
<section class="hero">
  <h1>Stock Replay</h1>
  <p class="muted">
    一个面向所有人的股票复盘记录与分享平台。
  </p>
  <p>
    MVP 采用 Rust + Axum + Askama + SQLx + PostgreSQL 的服务端渲染方案。
  </p>
</section>

<section class="card">
  <h2>最新公开复盘</h2>
  <div class="list">
    {% for replay in replays %}
    <article class="card">
      <div class="pill">{{ replay.market_sentiment }}</div>
      <h3><a href="/replays/{{ replay.id }}">{{ replay.title }}</a></h3>
      <p class="muted">
        {{ replay.trade_date }} · {{ replay.author_name }} · 👍 {{ replay.like_count }} · 💬 {{ replay.comment_count }}
      </p>
    </article>
    {% endfor %}
    {% if replays.len() == 0 %}
    <p class="muted">还没有公开复盘，先创建第一篇吧。</p>
    {% endif %}
  </div>
</section>
{% endblock %}

```

---
## `/apps/web/templates/replays/index.html`

```html
{% extends "base.html" %}

{% block title %}复盘广场{% endblock %}

{% block content %}
<section class="card">
  <h1>复盘广场</h1>
  <p class="muted">按交易日倒序展示公开复盘。</p>
</section>

<div class="list">
  {% for replay in replays %}
  <article class="card">
    <div class="pill">{{ replay.market_sentiment }}</div>
    <h2><a href="/replays/{{ replay.id }}">{{ replay.title }}</a></h2>
    <p class="muted">
      {{ replay.trade_date }} · {{ replay.author_name }} · 👍 {{ replay.like_count }} · 💬 {{ replay.comment_count }}
    </p>
  </article>
  {% endfor %}
  {% if replays.len() == 0 %}
  <div class="card">
    <p class="muted">暂无数据。</p>
  </div>
  {% endif %}
</div>
{% endblock %}

```

---
## `/apps/web/templates/replays/detail.html`

```html
{% extends "base.html" %}

{% block title %}{{ replay.title }}{% endblock %}

{% block content %}
<article class="card">
  <div class="pill">{{ replay.market_sentiment }}</div>
  <h1>{{ replay.title }}</h1>
  <p class="muted">
    交易日：{{ replay.trade_date }} · 创建于 {{ replay.created_at }}
  </p>
</article>

<section class="card">
  <h2>大盘观点</h2>
  <pre>{{ replay.market_view }}</pre>
</section>

<section class="card">
  <h2>主线板块</h2>
  <pre>{{ replay.sector_focus }}</pre>
</section>

<section class="card">
  <h2>关注个股</h2>
  <pre>{{ replay.watchlist }}</pre>
</section>

<section class="card">
  <h2>交易计划</h2>
  <pre>{{ replay.plan }}</pre>
</section>

<section class="card">
  <h2>实际执行</h2>
  <pre>{{ replay.execution }}</pre>
</section>

<section class="card">
  <h2>复盘总结</h2>
  <pre>{{ replay.review }}</pre>
</section>
{% endblock %}

```

---
## `/apps/web/templates/simple_page.html`

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<section class="card">
  <h1>{{ heading }}</h1>
  <p>{{ body }}</p>
</section>
{% endblock %}

```

---
# Optional static scaffold
## `/apps/web/static/.gitkeep`

```text


```

---
# Suggested folder tree

> 这份目录树是按你笔记中的建议骨架展开的，核心包括 `app.rs`、`config.rs`、`error.rs`、`routes/`、`handlers/`、`services/`、`repos/`、`models/`、`views/`、`templates/`、`migrations/`、`docker-compose.yml`。[1]
```text
stock-replay/
├─ Cargo.toml
├─ docker-compose.yml
├─ migrations/
│  ├─ 0001_init.sql
│  └─ 0002_indexes.sql
└─ apps/
   └─ web/
      ├─ Cargo.toml
      ├─ src/
      │  ├─ main.rs
      │  ├─ app.rs
      │  ├─ config.rs
      │  ├─ state.rs
      │  ├─ error.rs
      │  ├─ routes/
      │  │  ├─ mod.rs
      │  │  ├─ pages.rs
      │  │  └─ api.rs
      │  ├─ handlers/
      │  │  ├─ mod.rs
      │  │  ├─ pages.rs
      │  │  └─ api.rs
      │  ├─ services/
      │  │  ├─ mod.rs
      │  │  └─ replay_service.rs
      │  ├─ repos/
      │  │  ├─ mod.rs
      │  │  ├─ replay_repo.rs
      │  │  ├─ stock_repo.rs
      │  │  └─ tag_repo.rs
      │  ├─ models/
      │  │  └─ mod.rs
      │  └─ views/
      │     ├─ mod.rs
      │     └─ templates.rs
      ├─ templates/
      │  ├─ base.html
      │  ├─ home.html
      │  ├─ simple_page.html
      │  └─ replays/
      │     ├─ index.html
      │     └─ detail.html
      └─ static/
         └─ .gitkeep

```

---
## What came from your note vs what I filled in

**Directly grounded in your note:**

- 架构：Rust 全栈起点 = **Axum + Askama + SQLx + PostgreSQL**
- 目录结构方向
- 页面路由
- JSON API
- 数据表结构
- `Cargo.toml` 依赖集合
- `main.rs` 的基础形态与中间件方向[1]

**I filled in as MVP scaffold because your note didn’t contain full

[Conversation link](https://app.saner.ai/conversations/06b74384-e85c-4371-b0d0-b86ec5e43ceb)
