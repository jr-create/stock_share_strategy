pub mod home;
pub mod auth;
pub mod replay;
pub mod stocks;
pub mod dashboard;
pub mod market;
pub mod api;

use axum::Router;
use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .merge(home::routes())
        .merge(auth::routes())
        .merge(replay::routes())
        .merge(stocks::routes())
        .merge(dashboard::routes())
        .merge(market::routes())
        .nest("/api", api::routes())
}
