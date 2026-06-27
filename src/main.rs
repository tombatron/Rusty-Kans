mod models;
mod errors;

use crate::models::Board;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use crate::errors::KanbanError;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Board(BoardArgs),
    Card(CardArgs),
    List(ListArgs),
    Show,
}

#[derive(Args)]
struct BoardArgs {
    #[command(subcommand)]
    command: BoardCommands,
}

#[derive(Subcommand)]
enum BoardCommands {
    Create {
        name: String,
    }
}

#[derive(Args)]
struct CardArgs {
    #[command(subcommand)]
    command: CardCommands,
}

#[derive(Subcommand)]
enum CardCommands {
    Add {
        list_id: u64,
        title: String,
        description: Option<String>,
    },
    Move {
        card_id: u64,
        to_list_id: u64,
    },
    Search {
        keyword: String,
    },
    Show {
        card_id: u64,
    },
}

#[derive(Args)]
struct ListArgs {
    #[command(subcommand)]
    command: ListCommands
}

#[derive(Subcommand)]
enum ListCommands {
    Add {
        name: String
    },
    Sort {
        id: u64
    },
}

fn handle_board_command(args: BoardArgs, board_reference: &mut Option<Board>) {
    let board_command = args.command;

    if board_reference.is_some() {
        println!("You can't create a board because one's already been created.");
        return;
    }

    match board_command {
        BoardCommands::Create { name } => {
            *board_reference = Some(Board::new(name));
        },
    }
}

fn handle_card_command(args: CardArgs, board_reference: &mut Option<Board>) {
    let card_command = args.command;

    let Some(board) = board_reference.as_mut() else {
        println!("You can't manipulate cards on the board, because the board doesn't exist.");
        return;
    };

    match card_command {
        CardCommands::Add { list_id, title, description} => {
            let new_card_id = board.add_card(list_id, title.as_str(), description);

            match new_card_id {
                Ok(id) => println!("Card \"{title}\" id ({id}, was created on list_id {list_id}.)"),
                Err(msg) => println!("{msg}")
            }
        },

        CardCommands::Move { card_id, to_list_id } => {
            let card_move_result = board.move_card(card_id, to_list_id);

            match card_move_result {
                Ok(()) => println!("Card {card_id} was moved to list id {to_list_id}."),
                Err(msg) => println!("{msg}")
            }
        },

        CardCommands::Search { keyword} => {
            let search_results = board.search(&keyword);

            if search_results.is_empty() {
                println!("No cards were found that contain the keyword `{}` in the title.", keyword);
            } else {
                println!("Search results:");

                for result in search_results {
                    print!("{}", result);
                }
            }
        },

        CardCommands::Show { card_id } => {
            let card = board.lists.values()
                .flat_map(|l| l.cards.iter())
                .find(|c| c.id == card_id);

            if let Some(card) = card {
                println!("{}", card);
            } else {
                println!("Card ID `{}` is not found.", card_id);
            }
        }
    }
}

fn handle_list_command(args: ListArgs, board_reference: &mut Option<Board>) {
    let list_command = args.command;

    let Some(board) = board_reference.as_mut() else {
        println!("You can't work with lists on a board, because the board doesn't exist.");
        return;
    };

    match list_command {
        ListCommands::Add { name } => {
            let new_list_id = board.add_list(name.as_str());

            println!("New list \"{name}\" created as id {new_list_id}");
        },
        ListCommands::Sort { id } => {
            let Some(sort_target_list) = board.lists.get_mut(&id) else {
                println!("Couldn't sort List ID `{}` because that list wasn't found.", id);
                return;
            };

            sort_target_list.sort_by_title();

            println!("List ID `{}` was sorted.", id);
        }
    }
}

fn handle_show_command(board_reference: &Option<Board>) {
    let Some(board) = board_reference else {
        println!("You can't show the board, because the board doesn't exist.");
        return;
    };

    println!("{}", board);
}

fn load_existing_board_configuration() -> Option<Board> {
    let file = File::open("board.json").ok()?; // Returns here if the file doesn't exist.
    let reader = BufReader::new(file); // If the file doesn't exist we wouldn't get here.
    serde_json::from_reader(reader).ok() // The type is grabbed from the method signature and Some is returned if there's no error.
}

fn save_board_configuration(board: &Board) -> Result<(), KanbanError> {
    let file_content = serde_json::to_string(board)?;

    fs::write("board.json", &file_content)?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let mut board: Option<Board> = load_existing_board_configuration();

    if board.is_some() {
        println!("Loaded board from disk.");
    }

    match cli.commands {
        Commands::Board(args) => handle_board_command(args, &mut board),
        Commands::Card(args) => handle_card_command(args, &mut board),
        Commands::List(args) => handle_list_command(args, &mut board),
        Commands::Show => handle_show_command(&board)
    }

    if let Some(board) = board {
        let save_result = save_board_configuration(&board);

        if let Err(save_error) = save_result {
            println!("There was an error persisting the board: {save_error}");
        }
    }
}
