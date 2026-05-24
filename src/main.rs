mod models;

use crate::models::Board;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    println!("kanban-rs v{version}");

    let mut first_board = Board::new(String::from("This is the first board"));

    let first_list_id = first_board.add_list(String::from("First list!"));
    let second_list_id = first_board.add_list(String::from("Second list!"));

    let _ = first_board.add_card(first_list_id, String::from("First card?"), Some(String::from("Hello from the first card!"))).unwrap();
    let _ = first_board.add_card(second_list_id, String::from("Second card?"), Some(String::from("Hello from the second card!"))).unwrap();
    let card_id_3 = first_board.add_card(second_list_id, String::from("Third card?"), Some(String::from("Hello from the third card!"))).unwrap();

    let first_list_length = first_board.lists.iter().find(|l| l.id == first_list_id).unwrap().cards.len();
    let second_list_length = first_board.lists.iter().find(|l| l.id == second_list_id).unwrap().cards.len();
    
    println!("List 1 has: {first_list_length}");
    println!("List 2 has: {second_list_length}");

    first_board.move_card(card_id_3, first_list_id).unwrap();
    
    println!("Move!");

    let first_list_length = first_board.lists.iter().find(|l| l.id == first_list_id).unwrap().cards.len();
    let second_list_length = first_board.lists.iter().find(|l| l.id == second_list_id).unwrap().cards.len();

    println!("List 1 has: {first_list_length}");
    println!("List 2 has: {second_list_length}");
}
