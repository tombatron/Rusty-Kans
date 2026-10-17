use crate::data;
use crate::errors::KanbanError;
use crate::handlers::CreateCardRequest;
use crate::models::Card;
use crate::state::{ApiDb, ApplicationState};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/{owner_database_id}/{board_id}/lists/{id}/cards", post(post_card))
        .route("/api/{owner_database_id}/{board_id}/lists/{list_id}/cards/{card_id}/move", post(post_move_card))
        .route("/api/{owner_database_id}/{board_id}/lists/{list_id}/cards/{card_id}", delete(delete_card))
        .route("/api/{owner_database_id}/{board_id}/cards/search", get(get_card_search))
        .route("/api/{owner_database_id}/{board_id}/cards/{card_id}", get(get_card_by_id))
        .route("/api/{owner_database_id}/{board_id}/cards", patch(patch_card))
}

pub async fn post_card(ApiDb(db): ApiDb, Path((_, board_id, list_id)): Path<(String, i64, i64)>, Json(card): Json<CreateCardRequest>) -> Result<Json<Value>, KanbanError> {
    let card_id = data::insert_card_api(db, board_id, list_id, &card.title, &card.description).await?;

    if card_id == 0 {
        return Err(KanbanError::ListNotFound(list_id));
    }

    Ok(Json(json!({"card_id": card_id})))
}

pub async fn post_move_card(ApiDb(db): ApiDb, Path((_, board_id, list_id, card_id)): Path<(String, i64, i64, i64)>) -> Result<StatusCode, KanbanError> {
    let result = data::update_card_list_api(db, board_id, list_id, card_id).await?;

    if result == 0 {
        return Err(KanbanError::CardNotFound(card_id));
    }

    Ok(StatusCode::OK)
}

pub async fn get_card_by_id(ApiDb(db): ApiDb, Path((_, board_id, card_id)): Path<(String, i64, i64)>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_card_api(db, board_id, card_id).await?;

    Ok(Json(json!(result)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    keyword: String,
}

pub async fn get_card_search(ApiDb(db): ApiDb, Query(search_query): Query<SearchQuery>, Path((_, board_id)): Path<(String, i64)>) -> Result<Json<Value>, KanbanError> {
    let search_result = data::get_cards_by_title_submatch_api(db, board_id, search_query.keyword).await?;

    let cards = search_result.iter().map(|c| json!(c)).collect();

    Ok(Json(cards))
}

pub async fn delete_card(ApiDb(db): ApiDb, Path((_, board_id, card_id)): Path<(String, i64, i64)>) -> Result<StatusCode, KanbanError> {
    data::delete_card_api(db, board_id, card_id).await?;

    Ok(StatusCode::OK)
}

pub async fn patch_card(ApiDb(db): ApiDb, Path((_, board_id)): Path<(String, i64)>, Json(card): Json<Card>) -> Result<StatusCode, KanbanError> {
    let result = data::update_card_api(db, board_id, card.id, card.title, card.description).await?;

    if result != 1 {
        return Err(KanbanError::CardNotFound(card.id));
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use crate::data;
    use crate::handlers::CreateCardRequest;
    use crate::handlers::api::cards::*;
    use axum::Json;
    use axum::extract::Path;
    use axum::http::StatusCode;
    use sqlx::SqlitePool;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_card_adds_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let request = Json(CreateCardRequest {
            title: "This is a new card.".to_string(),
            description: Some("This is a nice description.".to_string()),
        });

        let response = post_card(db, Path(("".to_string(), 1, 1)), request).await.unwrap().0;

        assert_eq!(19, response["card_id"]);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_move_card_moves_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = post_move_card(db.clone(), Path(("".to_string(), 1, 2, 1))).await.unwrap();

        let card = data::get_card(db.0, 1).await.unwrap();

        assert_eq!(StatusCode::OK, response);
        assert_eq!(2, card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_card_by_id_gets_card_by_id(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = get_card_by_id(db, Path(("".to_string(), 1, 1))).await.unwrap().0;

        assert_eq!(1, response["id"]);
        assert_eq!("Card 1", response["title"]);
        assert_eq!("This is a description", response["description"]);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_card_by_search_does_that(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        // This should match every card on the board, unlike the non-api version which would search
        // all boards in the database. Not sure this should even be a method... **shrug**
        let request = Query(SearchQuery { keyword: "Card".to_string() });

        let response = get_card_search(db, request, Path(("".to_string(), 1))).await.unwrap().0;

        assert_eq!(6, response.as_array().unwrap().len());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn delete_card_does_it(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = delete_card(db.clone(), Path(("".to_string(), 1, 1))).await.unwrap();

        let card = data::get_card(db.0, 1).await;

        assert_eq!(StatusCode::OK, response);
        assert!(card.is_err()); // This should be an error because we just deleted it.

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn patch_card_updates_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let request = Json(Card {
            id: 1,
            list_id: 1,
            title: "UPDATED".to_string(),
            description: Some("UPDATED".to_string()),
            sort_order: None,
        });

        let _ = patch_card(db.clone(), Path(("".to_string(), 1)), request).await.unwrap();

        let updated_card = data::get_card(db.0, 1).await.unwrap();

        assert_eq!(1, updated_card.id);
        assert_eq!(1, updated_card.list_id);
        assert_eq!("UPDATED", updated_card.title);
        assert_eq!("UPDATED", updated_card.description.unwrap());

        Ok(())
    }
}
