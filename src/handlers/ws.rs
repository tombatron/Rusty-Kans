use crate::state::ApplicationState;
use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new().route("/ws", get(get_ws_handler))
}

pub async fn get_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApplicationState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: ApplicationState) {
    let mut rx = state.tx.subscribe();

    while let Ok(event) = rx.recv().await {
        let html = event
            .render()
            .map_err(|e| format!("<div>{e}</div>"))
            .unwrap();
        if socket.send(Message::Text(html.into())).await.is_err() {
            break;
        }
    }
}
