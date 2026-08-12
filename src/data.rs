use crate::errors::KanbanError;
use crate::models::{Board, BoardWithCards, Card, List, ListWithCards, Status};
use sqlx::{AssertSqlSafe, SqlitePool};

pub async fn get_card(db: &SqlitePool, card_id: u64) -> Result<Card, KanbanError> {
    Ok(sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;",
    )
    .bind(card_id as i64)
    .fetch_optional(db)
    .await?
    .ok_or(KanbanError::CardNotFound(card_id))?)
}

pub async fn get_cards_by_title_submatch(
    db: &SqlitePool,
    query: String,
) -> Result<Vec<Card>, KanbanError> {
    Ok(sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE title LIKE ?;",
    )
    .bind(format!("%{}%", query))
    .fetch_all(db)
    .await?)
}

pub async fn insert_board(db: &SqlitePool, board_name: &String) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO boards (name) VALUES (?);")
        .bind(board_name)
        .execute(db)
        .await?;

    let board_id = result.last_insert_rowid();

    Ok(board_id as u64)
}

pub async fn get_board(db: &SqlitePool, board_id: u64) -> Result<Board, KanbanError> {
    Ok(
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
            .bind(board_id as i64)
            .fetch_optional(db)
            .await?
            .ok_or(KanbanError::BoardNotFound(board_id))?,
    )
}

pub async fn get_all_boards(db: &SqlitePool) -> Result<Vec<Board>, KanbanError> {
    Ok(
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards;")
            .fetch_all(db)
            .await?
    )
}

pub async fn get_board_with_lists(
    db: &SqlitePool,
    board_id: u64,
) -> Result<BoardWithCards, KanbanError> {
    let board = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_optional(db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?;

    let mut lists: Vec<List> =
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE board_id = ?")
            .bind(board_id as i64)
            .fetch_all(db)
            .await?;

    if !lists.is_empty() {
        let list_ids_placeholders = lists.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query_text = format!(
            "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id in ({})",
            list_ids_placeholders
        );
        let mut query = sqlx::query_as::<_, Card>(AssertSqlSafe(query_text));

        for id in &lists {
            query = query.bind(id.id as i64);
        }

        let cards: Vec<Card> = query.fetch_all(db).await?;

        for l in lists.iter_mut() {
            l.cards = cards
                .iter()
                .filter(|c| c.list_id == l.id)
                .cloned()
                .collect()
        }
    }

    let result = BoardWithCards { board, lists };

    Ok(result)
}

pub async fn delete_board(db: &SqlitePool, board_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .execute(db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn update_board(
    db: &SqlitePool,
    board_id: u64,
    name: String,
) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE boards SET name = ? WHERE board_id = ?;")
        .bind(&name)
        .bind(board_id as i64)
        .execute(db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn get_list_header(db: &SqlitePool, list_id: u64) -> Result<List, KanbanError> {
    Ok(
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE list_id = ?;")
            .bind(list_id as i64)
            .fetch_one(db)
            .await?,
    )
}

pub async fn get_list_with_card(
    db: &SqlitePool,
    list_id: u64,
) -> Result<ListWithCards, KanbanError> {
    let list =
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE list_id = ?")
            .bind(list_id as i64)
            .fetch_optional(db)
            .await?
            .ok_or(KanbanError::ListNotFound(list_id))?;

    let cards = sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id = ?",
    )
    .bind(list_id as i64)
    .fetch_all(db)
    .await?;

    Ok(ListWithCards { list, cards })
}

pub async fn delete_card(db: &SqlitePool, card_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM cards WHERE card_id = ?")
        .bind(card_id as i64)
        .execute(db)
        .await?;
    
    Ok(result.rows_affected())
}

pub async fn insert_card(db: &SqlitePool, list_id: u64, title: &String, description: &Option<String>) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO cards (list_id, title, description) VALUES (?, ?, ?);")
        .bind(list_id as i64)
        .bind(title)
        .bind(description)
        .execute(db)
        .await?;
    
    Ok(result.last_insert_rowid() as u64)
}

pub async fn update_card(db: &SqlitePool, card_id: u64, title: String, description: Option<String>, status: Status) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE cards SET title = ?, description = ?, status = ? WHERE card_id = ?;")
        .bind(title)
        .bind(description)
        .bind(status)
        .bind(card_id as i64)
        .execute(db)
        .await?;
    
    Ok(result.rows_affected())
}

pub async fn update_card_list(db: &SqlitePool, target_list_id: u64, card_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE cards SET list_id = ? WHERE card_id = ?;")
        .bind(target_list_id as i64)
        .bind(card_id as i64)
        .execute(db)
        .await?;
    
    Ok(result.rows_affected())
}

pub async fn insert_list(db: &SqlitePool, board_id: u64, list_name: &String) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO lists (board_id, name) VALUES (?, ?);")
        .bind(board_id as i64)
        .bind(list_name)
        .execute(db)
        .await?;
    
    Ok(result.last_insert_rowid() as u64)
}

pub async fn delete_list(db: &SqlitePool, list_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM lists WHERE list_id = ?;")
        .bind(list_id as i64)
        .execute(db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn update_list(db: &SqlitePool, list_id: u64, name: String) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE lists SET name = ? WHERE list_id = ?")
        .bind(name)
        .bind(list_id as i64)
        .execute(db)
        .await?;

    Ok(result.rows_affected())
}
