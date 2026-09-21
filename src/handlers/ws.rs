use crate::state::{ApplicationState, UserBroadcast};
use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::sync::broadcast::Receiver;
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
    UserBroadcast(tx): UserBroadcast,
) -> Result<impl IntoResponse, KanbanError> {
    let rx = tx.subscribe();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, rx)))
}

async fn handle_socket(mut socket: WebSocket, mut rx: Receiver<SocketEvents>) {
    while let Ok(event) = rx.recv().await {
        let socket_message = match event {
            SocketEvents::CardMoved(card_move_event) => {
                card_move_event
                    .render()
                    .map_err(|e| format!("<div>{e}</div>"))
            },
            SocketEvents::ListsSorted(card_sort_update) => {
                card_sort_update
                    .render()
                    .map_err(|e| format!("<div>{e}</div>"))
            },
        };

        if socket.send(Message::Text(socket_message.unwrap_or_else(|e|e).into())).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{handlers::{web::tests::base_test_server, ws::SocketEvents}, models::CardMoveEvent, state::get_database_id};

    #[tokio::test]
    async fn websocket_will_broadcast_card_move_events() {
        // Step 1: Setup a basic test server.
        let (_, user_id, state, server) = base_test_server("/ws", true).await;

        // Step 2: Open a websocket and start listening. 
        let mut test_websocket = server.get_websocket("/ws").await.into_websocket().await;

        // Step 3: Create and send a CardMoveEvent through the transmission side of the web socket thats
        //         available through the application state. 
        let card_move_event = CardMoveEvent {
            card_id: 10000,
            to_list_id: 1,
        };

        // Step 4: Let's make sure that sending to the web socket actually worked. 
        let socket_id = get_database_id(user_id, "dev".to_string());
        let socket = state.tx_pool.get(&socket_id).unwrap().clone();

        let result = socket.send(SocketEvents::CardMoved(card_move_event));

        assert!(result.is_ok());

        // Step 5: Finally, let's make sure that we got the content we were looking for.
        test_websocket.assert_receive_text_contains("<turbo-stream action=\"remove\" target=\"card-10000\"></turbo-stream>").await;
    }
}