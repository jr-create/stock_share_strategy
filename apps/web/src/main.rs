mod config;
mod error;
mod routes;
mod scheduler;
mod state;

use std::net::SocketAddr;
use axum::Router;
use tower_http::trace::TraceLayer;
use tower_cookies::CookieManagerLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    
    let db = db::create_pool(&config.database_url).await?;
    
    db::run_migrations(&db).await?;
    
    let state = AppState::new(db);
    
    state.start_scheduler().await;
    
    let app = Router::new()
        .merge(routes::create_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let addr: SocketAddr = config.server_addr.parse()?;
    
    tracing::info!("服务器启动于 http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
