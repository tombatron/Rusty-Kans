use axum::Router;
use crate::state::ApplicationState;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
}