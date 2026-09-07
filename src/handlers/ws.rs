use crate::state::ApplicationState;
use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tower_sessions::Session;
use crate::csrf;
use crate::csrf::get_or_create_secret;
use crate::errors::KanbanError;
use crate::models::CardMoveEvent;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new().route("/ws", get(get_ws_handler))
}

pub async fn get_ws_handler(
    ws: WebSocketUpgrade,
    session: Session,
    State(state): State<ApplicationState>,
) -> Result<impl IntoResponse, KanbanError> {
    let secret = get_or_create_secret(&session).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, secret)))
}

async fn handle_socket(mut socket: WebSocket, state: ApplicationState, secret: [u8; 32]) {
    let mut rx = state.tx.subscribe();

    while let Ok(event) = rx.recv().await {
        let event = CardMoveEvent {
            csrf_token: csrf::mask(&secret),
            ..event
        };

        let html = event
            .render()
            .map_err(|e| format!("<div>{e}</div>"))
            .unwrap();
        if socket.send(Message::Text(html.into())).await.is_err() {
            break;
        }
    }
}
