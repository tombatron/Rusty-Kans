pub mod security;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::fmt::{Display, Formatter};
use askama::Template;

#[derive(Debug, FromRow)]
pub struct Board {
    #[sqlx(rename = "board_id")]
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Card {
    #[sqlx(rename = "card_id")]
    pub id: u64,
    pub list_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "    [{}] {}", self.id, self.title)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct List {
    #[sqlx(rename = "list_id")]
    pub id: u64,
    pub board_id: u64,
    pub name: String,
    #[sqlx(skip)]
    pub cards: Vec<Card>,
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "List: {} (id: {})", self.name, self.id)?;

        for card in &self.cards {
            writeln!(f, "{}", card)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "PascalCase")]
pub enum Status {
    Todo,
    Doing,
    Done,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => write!(f, "Todo"),
            Status::Doing => write!(f, "Doing"),
            Status::Done => write!(f, "Done")
        }
    }
}

impl PartialEq<&str> for Status {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().as_str() == *other
    }
}

#[derive(Debug, Clone, Template)]
#[template(path = "turbo_move_card.html")]
pub struct CardMoveEvent {
    pub card_id: u64,
    pub to_list_id: u64,
    pub card: Card,
}

pub struct BoardWithCards {
    pub board: Board,
    pub lists: Vec<List>,
}

pub struct ListWithCards {
    pub list: List,
    pub cards: Vec<Card>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn card_display_impl_validation() {
        let card = Card {
            id: 1,
            list_id: 2,
            title: "TITLE".to_string(),
            description: None,
            status: Status::Todo,
        };

        let string_result = card.to_string();

        assert_eq!("    [1] TITLE", string_result);
    }

    #[test]
    fn list_display_impl_validation() {
        let card = Card {
            id: 1,
            list_id: 1,
            title: "TITLE".to_string(),
            description: None,
            status: Status::Todo,
        };

        let list = List {
            id: 1,
            board_id: 1,
            name: "NAME".to_string(),
            cards: vec!(card)
        };

        let string_result = list.to_string();

        assert!(string_result.contains("List: NAME (id: 1)"));
        assert!(string_result.contains("    [1] TITLE"));
    }

    #[test_case(Status::Todo, "Todo".to_string())]
    #[test_case(Status::Doing, "Doing".to_string())]
    #[test_case(Status::Done, "Done".to_string())]
    fn status_display_impl_validation(status: Status, expected_result: String) {
        let result = status.to_string();

        assert_eq!(expected_result, result);
    }
}