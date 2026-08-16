use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use serde_json::{json, Value};
use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{create_list_common, CreateListRequest};
use crate::state::ApplicationState;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/lists/{id}", get(get_list_by_id))
        .route("/api/board/{board_id}/lists", post(post_list))
}

async fn get_list_by_id(
    State(state): State<ApplicationState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let result = data::get_list_with_cards(state.db, id).await?;

    let cards_json: Vec<Value> = result.cards
        .iter()
        .map(|c| json!(c))
        .collect();

    Ok(Json(json!({
        "list_id": id,
        "board_id": result.list.board_id,
        "name": result.list.name,
        "cards": cards_json
    })))
}

async fn post_list(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let created_list = create_list_common(State(state), board_id, list).await?;

    Ok(Json(json!(created_list)))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Path, State};
    use axum::Json;
    use sqlx::SqlitePool;
    use crate::handlers::api::lists::{get_list_by_id, post_list};
    use crate::handlers::CreateListRequest;
    use crate::handlers::tests::get_fake_application_state;

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_list_by_id_returns_list(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_list_by_id(state, Path(1)).await.unwrap().0;

        assert_eq!(1, response["list_id"]);
        assert_eq!(1, response["board_id"]);
        assert_eq!("First List", response["name"]);
        assert_eq!(3, response["cards"].as_array().unwrap().len());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_list_creates_list(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let request = CreateListRequest {
            name: "This is a totally new list.".to_string()
        };

        let response = post_list(state, Path(1), Json(request)).await.unwrap();

        assert_eq!(7, response["list_id"]);
        assert_eq!("This is a totally new list.", response["name"]);

        Ok(())
    }
}