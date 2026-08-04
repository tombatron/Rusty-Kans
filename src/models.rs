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
