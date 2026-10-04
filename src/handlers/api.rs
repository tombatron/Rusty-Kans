pub mod boards;
pub mod cards;
pub mod lists;

use crate::middleware::require_delegated_access;
use crate::state::ApplicationState;
use axum::Router;

pub fn get_router_configuration(state: ApplicationState) -> Router<ApplicationState> {
    Router::new()
        .merge(boards::get_router_configuration())
        .merge(cards::get_router_configuration())
        .merge(lists::get_router_configuration())
        .route_layer(axum::middleware::from_fn_with_state(state, require_delegated_access))
}
