use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List {
    pub id: u64,
    pub name: String,
    pub cards: Vec<Card>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    pub name: String,
    pub lists: Vec<List>,
    next_id: u64
}

impl Board {
    pub fn new(name: String) -> Self {
        Board {
            name,
            lists: vec![],
            next_id: 1
        }
    }

    pub fn add_list(&mut self, title: &str) -> u64 {
        let list_id = self.get_next_id();

        let list = List {
            id: list_id,
            name: title.to_string(),
            cards: vec![]
        };

        self.lists.push(list);

        list_id
    }

    pub fn add_card(&mut self, list_id: u64, title: &str, description: Option<String>) -> Result<u64, String> {
        let id = self.get_next_id();

        let target_list = match self.lists.iter_mut().find(|l| l.id == list_id) {
            None => return Err(format!("There is no list {list_id}.")),
            Some(list) => list
        };

        let new_card = Card {
            id,
            title: title.to_string(),
            description,
            status: Status::Todo
        };

        target_list.cards.push(new_card);

        Ok(id)
    }

    pub fn move_card(&mut self, card_id: u64, to_list_id: u64) -> Result<(), String> {
        let target_list_pos = self.lists.iter().position(|l| l.id == to_list_id);

        if target_list_pos.is_none() {
            return Err(format!("The target list {to_list_id} doesn't exist."));
        }

        let card = self.lists.iter_mut()
            .find_map(|list| {
               let pos = list.cards.iter().position(|c| c.id == card_id)?;
               Some(list.cards.remove(pos))
            });

        let target_list = self.lists.get_mut(target_list_pos.unwrap()).unwrap();

        match card {
            Some(card) => {
                target_list.cards.push(card);

                Ok(())
            },
            None => Err(format!("Cannot find the card id {card_id}"))
        }
    }

    fn get_next_id(&mut self) -> u64 {
        let result = self.next_id;

        self.next_id += 1;

        result
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Todo,
    Doing,
    Done
}