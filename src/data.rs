use crate::errors::KanbanError;
use crate::models::{Board, BoardWithListIds, Card, List, ListWithCards, User};
use sqlx::SqlitePool;

pub async fn insert_board(db: SqlitePool, board_name: &String) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO boards (name) VALUES (?);")
        .bind(board_name)
        .execute(&db)
        .await?;

    let board_id = result.last_insert_rowid();

    Ok(board_id as u64)
}

pub async fn get_board(db: SqlitePool, board_id: u64) -> Result<Board, KanbanError> {
    Ok(
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
            .bind(board_id as i64)
            .fetch_optional(&db)
            .await?
            .ok_or(KanbanError::BoardNotFound(board_id))?,
    )
}

pub async fn get_all_boards(db: SqlitePool) -> Result<Vec<Board>, KanbanError> {
    Ok(
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards ORDER BY board_id;")
            .fetch_all(&db)
            .await?
    )
}

pub async fn get_board_with_lists(
    db: SqlitePool,
    board_id: u64,
) -> Result<BoardWithListIds, KanbanError> {
    let board = get_board(db.clone(), board_id).await?;

    let rows: Vec<(u64,)> =
        sqlx::query_as("SELECT list_id FROM lists WHERE board_id = ? ORDER BY list_id;")
            .bind(board_id as i64)
            .fetch_all(&db)
            .await?;
    
    let list_ids = rows.into_iter().map(|(id,)| id).collect();

    let result = BoardWithListIds { board, list_ids };

    Ok(result)
}

pub async fn delete_board(db: SqlitePool, board_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn update_board(
    db: SqlitePool,
    board_id: u64,
    name: String,
) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE boards SET name = ? WHERE board_id = ?;")
        .bind(&name)
        .bind(board_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn get_list_header(db: SqlitePool, list_id: u64) -> Result<List, KanbanError> {
    Ok(
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE list_id = ?;")
            .bind(list_id as i64)
            .fetch_one(&db)
            .await?,
    )
}

pub async fn get_list_with_cards(
    db: SqlitePool,
    list_id: u64,
) -> Result<ListWithCards, KanbanError> {
    let list = get_list_header(db.clone(), list_id).await?;

    let cards = sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, sort_order FROM cards WHERE list_id = ? ORDER BY sort_order ASC",
    )
    .bind(list_id as i64)
    .fetch_all(&db)
    .await?;

    Ok(ListWithCards { list, cards })
}

pub async fn get_card(db: SqlitePool, card_id: u64) -> Result<Card, KanbanError> {
    Ok(sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, sort_order FROM cards WHERE card_id = ?;",
    )
        .bind(card_id as i64)
        .fetch_optional(&db)
        .await?
        .ok_or(KanbanError::CardNotFound(card_id))?)
}

pub async fn get_cards_by_title_submatch(
    db: SqlitePool,
    query: String,
) -> Result<Vec<Card>, KanbanError> {
    Ok(sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, sort_order FROM cards WHERE title LIKE ?;",
    )
        .bind(format!("%{}%", query))
        .fetch_all(&db)
        .await?)
}

pub async fn delete_card(db: SqlitePool, card_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM cards WHERE card_id = ?")
        .bind(card_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn insert_card(db: SqlitePool, list_id: u64, title: &String, description: &Option<String>) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO cards (list_id, title, description) VALUES (?, ?, ?);")
        .bind(list_id as i64)
        .bind(title)
        .bind(description)
        .execute(&db)
        .await?;

    Ok(result.last_insert_rowid() as u64)
}

pub async fn update_card(db: SqlitePool, card_id: u64, title: String, description: Option<String>) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE cards SET title = ?, description = ? WHERE card_id = ?;")
        .bind(title)
        .bind(description)
        .bind(card_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn update_card_list(db: SqlitePool, target_list_id: u64, card_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE cards SET list_id = ? WHERE card_id = ?;")
        .bind(target_list_id as i64)
        .bind(card_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn insert_list(db: SqlitePool, board_id: u64, list_name: &String) -> Result<u64, KanbanError> {
    let result = sqlx::query("INSERT INTO lists (board_id, name) VALUES (?, ?);")
        .bind(board_id as i64)
        .bind(list_name)
        .execute(&db)
        .await?;

    Ok(result.last_insert_rowid() as u64)
}

pub async fn delete_list(db: SqlitePool, list_id: u64) -> Result<u64, KanbanError> {
    let result = sqlx::query("DELETE FROM lists WHERE list_id = ?;")
        .bind(list_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn update_list(db: SqlitePool, list_id: u64, name: String) -> Result<u64, KanbanError> {
    let result = sqlx::query("UPDATE lists SET name = ? WHERE list_id = ?")
        .bind(name)
        .bind(list_id as i64)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

pub async fn get_user(db: SqlitePool, user_id: i64, source: String) -> Result<Option<User>, KanbanError> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT user_id, source, oauth_login, display_name, avatar_url FROM Users WHERE user_id = $1 and source = $2;")
        .bind(user_id)
        .bind(source)
        .fetch_optional(&db)
        .await?)
}

pub async fn upsert_user(db: SqlitePool, user: User) -> Result<u64, KanbanError> {
    let upsert_query = r#"
    INSERT INTO Users (user_id, source, oauth_login, display_name, avatar_url)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (user_id, source)
    DO UPDATE SET oauth_login = $3, display_name = $4, avatar_url = $5;
    "#;

    let result = sqlx::query(upsert_query)
        .bind(user.user_id as i64)
        .bind(user.source)
        .bind(user.oauth_login)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .execute(&db)
        .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use crate::data;
    use crate::models::{Board, User};
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn insert_board_returns_new_id(pool: SqlitePool) -> sqlx::Result<()> {
        let test_board_name = String::from("test is a test");

        let result = data::insert_board(pool.clone(), &test_board_name).await.unwrap();

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
        let result = data::get_board(pool, 1).await.unwrap();

        assert_eq!(1, result.id);
        assert_eq!(String::from("Whatever"), result.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_all_boards_returns_all_boards(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_all_boards(pool).await.unwrap();

        assert_eq!(3, result.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_board_with_lists_returns_boards_with_lists(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_board_with_lists(pool, 3).await.unwrap();

        assert_eq!("A third board?!", result.board.name);
        assert_eq!(2, result.list_ids.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_board_deletes_board(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_board(pool,1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_board_updates_board(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::update_board(pool.clone(), 1, String::from("This is a test!!!")).await.unwrap();

        let updated_board = data::get_board(pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!("This is a test!!!", updated_board.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_list_header_returns_list_header(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_list_header(pool, 1).await.unwrap();

        assert_eq!(1, result.id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_list_with_cards_returns_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_list_with_cards(pool, 1).await.unwrap();

        assert_eq!(1, result.list.id);
        assert_eq!(3, result.cards.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_card_returns_card(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::get_card(pool, 1).await.unwrap();

        assert_eq!(1, result.id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_cards_by_title_submatch_returns_expected_cards(pool: SqlitePool) -> sqlx::Result<()> {
        let results = data::get_cards_by_title_submatch(pool, String::from("card")).await.unwrap();

        assert_eq!(18, results.len());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_card_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_card(pool, 1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn insert_card_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let title = String::from("This is just a test");
        let description: Option<String> = Some(String::from("whatever"));

        let result = data::insert_card(pool, 1, &title, &description).await.unwrap();

        assert_eq!(19, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_card_updates_card(pool: SqlitePool) -> sqlx::Result<()> {
        let new_title = String::from("New Title");
        let new_description = String::from("New Description");

        let result = data::update_card(pool.clone(), 1, new_title, Some(new_description)).await.unwrap();
        let updated_card = data::get_card(pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!("New Title", updated_card.title);
        assert_eq!("New Description", updated_card.description.unwrap().to_string());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_card_list_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::update_card_list(pool.clone(), 2, 1).await.unwrap();

        let updated_card = data::get_card(pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!(2, updated_card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn insert_list_does_that(pool: SqlitePool) -> sqlx::Result<()> {
        let new_list_name = String::from("This is a new list");

        let result = data::insert_list(pool.clone(), 1, &new_list_name).await.unwrap();

        let board = data::get_board_with_lists(pool, 1).await.unwrap();
        let new_list = board.list_ids.iter().find(|l| **l == result).unwrap();

        assert_eq!(7, result);
        assert_eq!(result, *new_list);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_list_deletes_list(pool: SqlitePool) -> sqlx::Result<()> {
        let result = data::delete_list(pool, 1).await.unwrap();

        assert_eq!(1, result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn update_list_does_the_thing(pool: SqlitePool) -> sqlx::Result<()> {
        let new_list_name = String::from("This is a new list name.");
        let result = data::update_list(pool.clone(), 1, new_list_name.clone()).await.unwrap();

        let updated_list = data::get_list_header(pool, 1).await.unwrap();

        assert_eq!(1, result);
        assert_eq!(new_list_name, updated_list.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn upsert_user_will_add_a_new_user(pool: SqlitePool) -> sqlx::Result<()> {
        let test_user = User {
            user_id: -10,
            source: "test".to_string(),
            oauth_login: "whatever".to_string(),
            display_name: Some("some name".to_string()),
            avatar_url: Some("this is a bogus value".to_string())
        };

        let upsert_result = data::upsert_user(pool, test_user).await.unwrap();
        
        assert_eq!(1, upsert_result);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn upsert_will_update_a_user_if_they_already_exist(pool: SqlitePool) -> sqlx::Result<()> {
        let test_user = User {
            user_id: -10000,
            source: "dev".to_string(),
            oauth_login: "updated oauth login".to_string(),
            display_name: Some("updated display name".to_string()),
            avatar_url: Some("updated avatar url".to_string())
        }; 

        let upsert_result = data::upsert_user(pool.clone(), test_user).await.unwrap();

        assert_eq!(1, upsert_result);

        let upserted_user = data::get_user(pool.clone(), -10000, "dev".to_string()).await.unwrap().unwrap();

        assert_eq!("updated oauth login", upserted_user.oauth_login);
        assert_eq!("updated display name", upserted_user.display_name.unwrap());
        assert_eq!("updated avatar url", upserted_user.avatar_url.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_user_will_get_an_existing_user(pool: SqlitePool) -> sqlx::Result<()> {
        let test_user = data::get_user(pool, -10000, "dev".to_string()).await.unwrap().unwrap();

        assert_eq!(-10000, test_user.user_id);
        assert_eq!("dev", test_user.source);
        assert_eq!("dev_login", test_user.oauth_login);
        assert_eq!("test user", test_user.display_name.unwrap());
        assert_eq!("http://example.com/whatever.gif", test_user.avatar_url.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn get_user_will_return_an_empty_result_if_missing_user(pool: SqlitePool) -> sqlx::Result<()> {
        let nonexistant_user = data::get_user(pool, -9000000, "whatever".to_string()).await.unwrap();

        assert!(nonexistant_user.is_none());

        Ok(())
    }
}