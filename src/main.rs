mod models;

use crate::models::{Board, Card, List, Status};

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    println!("kanban-rs v{version}");

    let first_card = Card {
        id: 1,
        title: String::from("First Card"),
        description: Some(String::from("This is the first card")),
        status: Status::Todo,
    };

    let first_list = List {
        id: 1,
        name: String::from("This is the first list."),
        cards: vec![first_card],
    };

    let first_board = Board {
        name: String::from("This is the first board."),
        lists: vec![first_list],
    };

    println!("{first_board:?}");
}
