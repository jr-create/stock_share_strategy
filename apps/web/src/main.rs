mod config;
mod error;
mod state;
mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tower_cookies::CookieManagerLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Connecting to database...");
    
    // Database connection with retry logic
    let mut retries = 0;
    let max_retries = 30;
    let db = loop {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                tracing::info!("Successfully connected to database");
                break pool;
            }
            Err(e) if retries < max_retries => {
                retries += 1;
                tracing::warn!(
                    "Failed to connect to database (attempt {}/{}): {}",
                    retries,
                    max_retries,
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to connect to database after {} attempts: {}",
                    max_retries,
                    e
                ))
            }
        }
    };

    tracing::info!("Running database migrations...");
    sqlx::migrate!("../../migrations")
        .run(&db)
        .await?;

    let state = AppState::new(db);

    let app = Router::new()
        .merge(routes::create_routes())
        .nest_service("/static", ServeDir::new("apps/web/static"))
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = config.server_addr.parse()?;
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
