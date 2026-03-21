pub mod replays;
pub mod stocks;
pub mod tags;
pub mod sync;

use axum::Router;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(replays::routes())
        .merge(stocks::routes())
        .merge(tags::routes())
        .merge(sync::routes())
}
