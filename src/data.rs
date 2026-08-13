use crate::errors::KanbanError;
use crate::models::{Board, BoardWithCards, Card, List, ListWithCards, Status};
use sqlx::{AssertSqlSafe, SqlitePool};

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
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards ORDER BY board_id;")
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
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE board_id = ? ORDER BY list_id;")
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

pub async fn get_list_with_cards(
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

#[cfg(test)]
mod tests {
    use crate::data;
    use crate::models::{Board, Status};
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn insert_board_returns_new_id(pool: SqlitePool) -> sqlx::Result<()> {
        let test_board_name = String::from("test is a test");

        let result = data::insert_board(&pool, &test_board_name).await.unwrap();

        let inserted_item = sqlx::query_as::<_, Board>("SELECT * FROM boards WHERE board_id = ?;")
            .bind(result as i64)
            .fetch_one(&pool)
            .await?;

        assert!(result > 0);
        assert_eq!(test_board_name, inserted_item.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_board_returns_result(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_board(&pool, 1).await.unwrap();

        assert_eq!(1, result.id);
        assert_eq!(String::from("Whatever"), result.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_all_boards_returns_all_boards(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_all_boards(&pool).await.unwrap();

        assert_eq!(3, result.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_board_with_lists_returns_boards_with_lists(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_board_with_lists(&pool, 3).await.unwrap();

        assert_eq!("A third board?!", result.board.name);
        assert_eq!(2, result.lists.len());
        assert_eq!("Card 12", result.lists[0].cards[0].title);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_board_deletes_board(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_board(&pool,1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_board_updates_board(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::update_board(&pool, 1, String::from("This is a test!!!")).await.unwrap();

        let updated_board = data::get_board(&pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!("This is a test!!!", updated_board.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_list_header_returns_list_header(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_list_header(&pool, 1).await.unwrap();

        assert_eq!(1, result.id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_list_with_cards_returns_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_list_with_cards(&pool, 1).await.unwrap();

        assert_eq!(1, result.list.id);
        assert_eq!(3, result.cards.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_card_returns_card(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_card(&pool, 1).await.unwrap();

        assert_eq!(1, result.id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_cards_by_title_submatch_returns_expected_cards(pool: SqlitePool) -> sqlx::Result<()> {
        let results = data::get_cards_by_title_submatch(&pool, String::from("card")).await.unwrap();

        assert_eq!(18, results.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_card_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_card(&pool, 1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn insert_card_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let title = String::from("This is just a test");
        let description: Option<String> = Some(String::from("whatever"));

        let result = data::insert_card(&pool, 1, &title, &description).await.unwrap();

        assert_eq!(19, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_card_updates_card(pool: SqlitePool) -> sqlx::Result<()> {
        let new_title = String::from("New Title");
        let new_description = String::from("New Description");
        let new_status = Status::Doing;

        let result = data::update_card(&pool, 1, new_title, Some(new_description), new_status).await.unwrap();
        let updated_card = data::get_card(&pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!("New Title", updated_card.title);
        assert_eq!("New Description", updated_card.description.unwrap().to_string());
        assert!(matches!(updated_card.status, Status::Doing));

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_card_list_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::update_card_list(&pool, 2, 1).await.unwrap();

        let updated_card = data::get_card(&pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!(2, updated_card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn insert_list_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let new_list_name = String::from("This is a new list");

        let result = data::insert_list(&pool, 1, &new_list_name).await.unwrap();

        let board = data::get_board_with_lists(&pool, 1).await.unwrap();
        let new_list = board.lists.iter().find(|l| l.id == result).unwrap();

        assert_eq!(7, result);
        assert_eq!(result, new_list.id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_list_deletes_list(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_list(&pool, 1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_list_does_the_thing(pool: SqlitePool) -> sqlx::Result<()> {
        let new_list_name = String::from("This is a new list name.");
        let result = data::update_list(&pool, 1, new_list_name.clone()).await.unwrap();

        let updated_list = data::get_list_header(&pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!(new_list_name, updated_list.name);

        Ok(())
    }
}