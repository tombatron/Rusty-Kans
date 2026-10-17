use crate::data;
use crate::errors::KanbanError;
use crate::handlers::CreateListRequest;
use crate::state::{ApiDb, ApplicationState};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/{owner_database_id}/{board_id}/lists/{list_id}", get(get_list_by_id))
        .route("/api/{owner_database_id}/{board_id}/board/lists", post(post_list))
}

async fn get_list_by_id(ApiDb(db): ApiDb, Path((_, board_id,list_id)): Path<(String, i64, i64)>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_list_with_cards_api(db, board_id, list_id).await?;

    let cards_json: Vec<Value> = result.cards
        .iter()
        .map(|c| json!(c))
        .collect();

    Ok(Json(json!({
        "list_id": list_id,
        "board_id": result.list.board_id,
        "name": result.list.name,
        "cards": cards_json
    })))
}

async fn post_list(ApiDb(db): ApiDb, Path((_, board_id)): Path<(String, i64)>, Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let list_id = data::insert_list(db, board_id, &list.name).await?;

    let created_list = json!({
        "list_id": list_id as i64,
        "name": list.name
    });

    Ok(Json(created_list))
}

#[cfg(test)]
mod tests {
    use crate::handlers::CreateListRequest;
    use crate::handlers::api::lists::{get_list_by_id, post_list};
    use crate::state::ApiDb;
    use axum::Json;
    use axum::extract::Path;
    use sqlx::SqlitePool;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_list_by_id_returns_list(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let response = get_list_by_id(db, Path(("".to_string(), 1, 1))).await.unwrap().0;

        assert_eq!(1, response["list_id"]);
        assert_eq!(1, response["board_id"]);
        assert_eq!("First List", response["name"]);
        assert_eq!(3, response["cards"].as_array().unwrap().len());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_list_creates_list(db: SqlitePool) -> sqlx::Result<()> {
        let db = ApiDb(db);

        let request = CreateListRequest {
            name: "This is a totally new list.".to_string()
        };

        let response = post_list(db, Path(("".to_string(), 1 )), Json(request)).await.unwrap();

        assert_eq!(7, response["list_id"]);
        assert_eq!("This is a totally new list.", response["name"]);

        Ok(())
    }
}