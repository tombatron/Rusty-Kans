use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{
    CreateCardRequest, create_card_common, delete_card_common, move_card_common, patch_card_common,
};
use crate::models::Card;
use crate::state::{ApplicationState, ApiDb};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/lists/{id}/cards", post(post_card))
        .route(
            "/api/lists/{list_id}/cards/{card_id}/move",
            post(post_move_card),
        )
        .route("/api/lists/{list_id}/cards/{card_id}", delete(delete_card))
        .route("/api/cards/search", get(get_card_search))
        .route("/api/cards/{card_id}", get(get_card_by_id))
        .route("/api/cards/{card_id}", patch(patch_card))
}

pub async fn post_card(
    ApiDb(db): ApiDb,
    Path(list_id): Path<u64>,
    Json(card): Json<CreateCardRequest>,
) -> Result<Json<Value>, KanbanError> {
    let card = create_card_common(db, list_id, card).await?;

    Ok(Json(json!(card)))
}

pub async fn post_move_card(
    ApiDb(db): ApiDb,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<StatusCode, KanbanError> {
    move_card_common(db, list_id, card_id).await?;

    Ok(StatusCode::OK)
}

pub async fn get_card_by_id(
    ApiDb(db): ApiDb,
    Path(card_id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let result = data::get_card(db, card_id).await?;

    Ok(Json(json!(result)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    keyword: String,
}

pub async fn get_card_search(
    ApiDb(db): ApiDb,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Value>, KanbanError> {
    let search_result = data::get_cards_by_title_submatch(db, search_query.keyword).await?;

    let cards = search_result.iter().map(|c| json!(c)).collect();

    Ok(Json(cards))
}

pub async fn delete_card(
    ApiDb(db): ApiDb,
    Path((_list_id, card_id)): Path<(u64, u64)>,
) -> Result<StatusCode, KanbanError> {
    delete_card_common(db, card_id).await?;

    Ok(StatusCode::OK)
}

pub async fn patch_card(
    ApiDb(db): ApiDb,
    Path(card_id): Path<u64>,
    Json(card): Json<Card>,
) -> Result<Redirect, KanbanError> {
    patch_card_common(db, card).await?;

    Ok(Redirect::to(format!("/api/cards/{card_id}").as_str()))
}

#[cfg(test)]
mod tests {
    use crate::data;
    use crate::handlers::CreateCardRequest;
    use crate::handlers::api::cards::*;
    use crate::models::Status;
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

        let response = post_card(db, Path(1), request).await.unwrap().0;

        assert_eq!(19, response["id"]);
        assert_eq!(1, response["list_id"]);
        assert_eq!("This is a new card.", response["title"]);
        assert_eq!("This is a nice description.", response["description"]);
        assert_eq!("Todo", response["status"]);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_move_card_moves_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = post_move_card(db.clone(), Path((2, 1))).await.unwrap();

        let card = data::get_card(db.0, 1).await.unwrap();

        assert_eq!(StatusCode::OK, response);
        assert_eq!(2, card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_card_by_id_gets_card_by_id(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = get_card_by_id(db, Path(1)).await.unwrap().0;

        assert_eq!(1, response["id"]);
        assert_eq!("Card 1", response["title"]);
        assert_eq!("This is a description", response["description"]);
        assert_eq!("Todo", response["status"]);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_card_by_search_does_that(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let request = Query(SearchQuery {
            keyword: "Card 1".to_string(),
        });

        let response = get_card_search(db, request).await.unwrap().0;

        assert_eq!(9, response.as_array().unwrap().len());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn delete_card_does_it(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = delete_card(db.clone(), Path((1, 1))).await.unwrap();

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
            status: Status::Done,
        });

        let response = patch_card(db.clone(), Path(1), request).await.unwrap();

        let updated_card = data::get_card(db.0, 1).await.unwrap();

        assert_eq!("/api/cards/1", response.location());
        assert_eq!(1, updated_card.id);
        assert_eq!(1, updated_card.list_id);
        assert_eq!("UPDATED", updated_card.title);
        assert_eq!("UPDATED", updated_card.description.unwrap());
        assert!(matches!(updated_card.status, Status::Done));

        Ok(())
    }
}
