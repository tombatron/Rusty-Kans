use crate::errors::KanbanError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub trait HasId {
    fn id(&self) -> u64;
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
}

impl HasId for Card {
    fn id(&self) -> u64 {
        self.id
    }
}

impl HasId for &Card {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "    [{}] {}", self.id, self.title)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    pub id: u64,
    pub name: String,
    pub cards: Vec<Card>,
}

impl HasId for List {
    fn id(&self) -> u64 {
        self.id
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    pub name: String,
    pub lists: HashMap<u64, List>,
    next_id: u64,
}

impl Board {
    pub fn new(name: String) -> Self {
        Board {
            name,
            lists: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_list(&mut self, title: &str) -> u64 {
        let list_id = self.get_next_id();

        let list = List {
            id: list_id,
            name: title.to_string(),
            cards: vec![],
        };

        self.lists.insert(list_id, list); // Should we evaluate the resulting option?

        list_id
    }

    pub fn add_card(
        &mut self,
        list_id: u64,
        title: &str,
        description: Option<String>,
    ) -> Result<u64, KanbanError> {
        let id = self.get_next_id();

        if let Some(target_list) = self.lists.get_mut(&list_id) {
            let new_card = Card {
                id,
                title: title.to_string(),
                description,
                status: Status::Todo,
            };

            target_list.cards.push(new_card);

            return Ok(id);
        }

        Err(KanbanError::ListNotFound(list_id))
    }

    pub fn move_card(&mut self, card_id: u64, to_list_id: u64) -> Result<(), KanbanError> {
        let card = self.lists.values_mut().find_map(|list| {
            let pos = list.cards.iter().position(|c| c.id == card_id)?;
            Some(list.cards.remove(pos))
        });

        let target_list = match self.lists.get_mut(&to_list_id) {
            None => return Err(KanbanError::ListNotFound(to_list_id)),
            Some(tl) => tl,
        };

        match card {
            Some(card) => {
                target_list.cards.push(card);

                Ok(())
            }
            None => Err(KanbanError::CardNotFound(card_id)),
        }
    }

    pub fn search(&self, keyword: &str) -> Vec<&Card> {
        self.lists
            .values()
            .flat_map(|l| l.cards.iter())
            .filter(|c| c.title.contains(keyword))
            .collect()
    }

    pub fn find_by_id<'a, T: HasId>(&self, items: &'a [T], id: u64) -> Option<&'a T> {
        items.iter().find(|item| item.id() == id)
    }

    fn get_next_id(&mut self) -> u64 {
        let result = self.next_id;

        self.next_id += 1;

        result
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Board: {}", self.name)?;

        for list in self.lists.values() {
            write!(f, "{}", list)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Todo,
    Doing,
    Done,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_a_list_increments_the_count() {
        let mut board = Board::new("Test".to_string());

        let initial_count = board.lists.len();

        board.add_list("whatever");

        let final_count = board.lists.len();

        assert_eq!(initial_count, 0);
        assert_eq!(final_count, 1);
    }

    #[test]
    fn can_add_a_card_to_a_list() {
        let mut board = Board::new("Test".to_string());
        let list_id = board.add_list("whatever");

        let card_add_result = board
            .add_card(
                list_id,
                "New test card",
                Some("some description".to_string()),
            )
            .expect("I guess the card didn't get added?");

        assert_ne!(card_add_result, 0);
    }

    #[test]
    fn can_not_add_a_card_to_a_non_existent_list() {
        let mut board = Board::new("Whatever".to_string());

        let card_add_result = board.add_card(67, "Bogus test card", None);

        assert_eq!(card_add_result, Err(KanbanError::ListNotFound(67)));
    }
}
