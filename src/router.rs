use crate::handlers::*;
use crate::ws::get_ws_handler;
use crate::state::ApplicationState;
use axum::routing::{get, post};
use axum::Router;
use crate::middleware::require_auth;

pub fn create_router(application_state: ApplicationState) -> Router<()> {
    let api = Router::new()
        .route("/api/board", get(get_board))
        .route("/api/lists/{id}", get(get_list_by_id))
        .route("/api/lists", post(post_list))
        .route("/api/lists/{id}/cards", post(post_card))
        .route("/api/lists/{list_id}/cards/{card_id}/move", post(post_move_card))
        .route("/api/cards/search", get(get_card_search))
        .route("/api/cards/{card_id}", get(get_card_by_id))
        .layer(axum::middleware::from_fn(require_auth));
    
    Router::new()
        // Web Routes
        .route("/", get(get_index))

        // Web Routes - lists
        .route("/lists", post(post_list_form))
        .route("/lists/{list_id}/cards/{card_id}/move", post(post_move_card_action))
        
        // Web Sockets
        .route("/ws", get(get_ws_handler))

        // Diagnostic Routes
        .route("/health", get(get_health))
        // --------------------------------------------------------------------------------
        .merge(api)
        .with_state(application_state)
}
