use std::fs;
use crate::models::Board;
use std::sync::{Arc, Mutex};

pub type ApplicationState = Arc<Mutex<Board>>;

pub fn create_application_state() -> ApplicationState {
    let board = load_existing_board_configuration()
        .unwrap_or_else(|| Board::new(String::from("Web board")));

    ApplicationState::new(Mutex::new(board))
}

fn load_existing_board_configuration() -> Option<Board> {
    let saved_board_configuration = fs::read_to_string("board.json").ok()?;
    serde_json::from_str(&saved_board_configuration).ok()
}
