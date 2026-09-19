use crate::state::ApplicationState;
use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::sync::broadcast::Receiver;
use tower_sessions::Session;
use crate::csrf;
use crate::csrf::get_or_create_secret;
use crate::errors::KanbanError;
use crate::models::{CardMoveEvent, CardSortUpdate};

#[derive(Clone)]
pub enum SocketEvents {
    CardMoved(CardMoveEvent),
    ListsSorted(CardSortUpdate),
}

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new().route("/ws", get(get_ws_handler))
}

pub async fn get_ws_handler(
    ws: WebSocketUpgrade,
    session: Session,
    State(state): State<ApplicationState>,
) -> Result<impl IntoResponse, KanbanError> {
    let secret = get_or_create_secret(&session).await?;
    let rx = state.tx.subscribe();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, rx, secret)))
}

async fn handle_socket(mut socket: WebSocket, mut rx: Receiver<SocketEvents>, secret: [u8; 32]) {
    while let Ok(event) = rx.recv().await {
        let socket_message = match event {
            SocketEvents::CardMoved(card_move_event) => {
                let event = CardMoveEvent {
                    csrf_token: csrf::mask(&secret),
                    ..card_move_event
                };

                event
                    .render()
                    .map_err(|e| format!("<div>{e}</div>"))
                    .unwrap()
            },
            SocketEvents::ListsSorted(card_sort_update) => {
                card_sort_update
                    .render()
                    .map_err(|e| format!("<div>{e}</div>"))
                    .unwrap()
            },
        };

        if socket.send(Message::Text(socket_message.into())).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{handlers::{web::tests::base_test_server, ws::SocketEvents}, models::{Card, CardMoveEvent}};

    #[tokio::test]
    async fn websocket_will_broadcast_card_move_events() {
        // Step 1: Setup a basic test server.
        let (token, state, server) = base_test_server("/ws", true).await;

        // Step 2: Open a websocket and start listening. 
        let mut test_websocket = server.get_websocket("/ws").await.into_websocket().await;

        // Step 3: Create and send a CardMoveEvent through the transmission side of the web socket thats
        //         available through the application state. 
        let card_move_event = CardMoveEvent {
            card_id: 10000,
            to_list_id: 1,
            card: Card {
                id: 10000,
                list_id: 1,
                title: "Whatever".to_string(),
                description: None,
                sort_order: None,
            },
            csrf_token: token,
        };

        // Step 4: Let's make sure that sending to the web socket actually worked. 
        let result = state.tx.send(SocketEvents::CardMoved(card_move_event));

        assert!(result.is_ok());

        // Step 5: Finally, let's make sure that we got the content we were looking for.
        test_websocket.assert_receive_text_contains("<turbo-stream action=\"remove\" target=\"card-10000\"></turbo-stream>").await;
    }
}