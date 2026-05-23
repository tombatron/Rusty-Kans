#[derive(Debug)]
pub struct Card {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status 
}

#[derive(Debug)]
pub struct List {
    pub id: u64,
    pub name: String,
    pub cards: Vec<Card>
}

#[derive(Debug)]
pub struct Board {
    pub name: String,
    pub lists: Vec<List>
}

#[derive(Debug)]
pub enum Status {
    Todo,
    Doing,
    Done
}