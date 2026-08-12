use crate::errors::KanbanError;
use sqlx::{AssertSqlSafe, SqlitePool};
use crate::models::{Board, BoardWithCards, Card, List};

pub async fn insert_board(db: &SqlitePool, board_name: &String) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO boards (name) VALUES (?);")
        .bind(board_name)
        .execute(db)
        .await?;

    let board_id = result.last_insert_rowid();
    
    Ok(board_id as u64)
}

pub async fn get_board(db: &SqlitePool, board_id: u64) -> Result<Board, KanbanError> {
    Ok(sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_optional(db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?)
}

pub async fn get_board_with_lists(db: &SqlitePool, board_id: u64) -> Result<BoardWithCards, KanbanError> {
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
    
    let result = BoardWithCards {
        board,
        lists,
    };
    
    Ok(result)
}

pub async fn delete_board(db: &SqlitePool, board_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .execute(db)
        .await?;
    
    Ok(result.rows_affected())
}

pub async fn update_board(db: &SqlitePool, board_id: u64, name: String) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE boards SET name = ? WHERE board_id = ?;")
           .bind(&name)
           .bind(board_id as i64)
           .execute(db)
           .await?;
    
    Ok(result.rows_affected())
}