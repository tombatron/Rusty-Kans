use crate::handlers::*;
use crate::state::ApplicationState;
use axum::routing::{get, post};
use axum::Router;

pub fn create_router(application_state: ApplicationState) -> Router<()> {
    Router::new()
        .route("/health", get(get_health))
        .route("/board", get(get_board))
        .route("/lists/{id}", get(get_list_by_id))
        .route("/lists", post(post_list))
        .route("/lists/{id}/cards", post(post_card))
        .route(
            "/lists/{list_id}/cards/{card_id}/move",
            post(post_move_card),
        )
        .route("/cards/search", get(get_card_search))
        .with_state(application_state)
}
