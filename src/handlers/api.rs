pub mod boards;
pub mod cards;
pub mod lists;

use crate::middleware::require_auth;
use crate::state::ApplicationState;
use axum::Router;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .merge(boards::get_router_configuration())
        .merge(cards::get_router_configuration())
        .merge(lists::get_router_configuration())
        .layer(axum::middleware::from_fn(require_auth))
}
