use crate::data;
use crate::errors::KanbanError;
use crate::models::{Board, Card, List};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))
        .route("/boards/{board_id}/edit", get(get_board_edit))
        .route("/boards/{board_id}/header", get(get_board_header))
        .route("/boards/{board_id}/rename", post(post_board_rename))
        .route("/boards/{board_id}/delete", post(post_board_delete))
}

#[derive(Debug, Deserialize)]
struct CreateBoardRequest {
    name: String,
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_board.html")]
struct NewBoardTemplate {
    id: u64,
    name: String
}

async fn post_board_form(State(state): State<ApplicationState>, Form(board): Form<CreateBoardRequest>) -> Result<TurboStream, KanbanError> {
    let board_id = data::insert_board(state.db, &board.name).await?;

    let template = NewBoardTemplate {
        id: board_id as u64,
        name: board.name,
    };

    Ok(TurboStream(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    board: Board,
    lists: Vec<List>,
}

pub async fn get_board(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>
) -> Result<Html<String>, KanbanError> {
    let board = data::get_board_with_lists(state.db, board_id).await?;

    let response_template = BoardTemplate {
        board: board.board,
        lists: board.lists,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title_edit.html")]
struct BoardHeaderEdit {
    board: Board
}

async fn get_board_edit(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(state.db, board_id).await?;

    let template = BoardHeaderEdit { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title.html")]
struct BoardHeader {
    board: Board
}

async fn get_board_header(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(state.db, board_id).await?;

    let template = BoardHeader { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
struct BoardRename {
    id: u64,
    name: String,
}

async fn post_board_rename(State(state): State<ApplicationState>, Path(board_id):Path<u64>, Form(board): Form<BoardRename>) -> Result<Redirect, KanbanError> {
    if board_id != board.id {
        return Err(KanbanError::RequestError(format!("Ambiguous board specified, path says `{}` and the form says `{}`", board_id, board.id)));
    }

    let result = data::update_board(state.db, board_id, board.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/boards/{}/header", board_id).as_str()))
}

async fn post_board_delete(State(state): State<ApplicationState>, Path(board_id):Path<u64>) -> Result<Redirect, KanbanError> {
    let result = data::delete_board(state.db, board_id).await?;

    if result != 1 {
        return Err(KanbanError::BoardNotFound(board_id));
    }

    Ok(Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use crate::handlers::tests::get_fake_application_state;
    use crate::handlers::web::boards::*;
    use crate::handlers::web::tests::*;
    use axum::Form;
    use axum::extract::{Path, State};
    use sqlx::SqlitePool;
    use test_case::test_case;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_form_creates_a_new_board(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let request = CreateBoardRequest {
            name: String::from("New Board Dude.")
        };

        let response = post_board_form(state, Form(request)).await.unwrap().0;

        assert!(response.contains("New Board Dude."));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_returns_a_board_page(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_board(state, Path(2)).await.unwrap().0;

        assert!(response.contains("Another Board"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_edit_returns_edit_form(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_board_edit(state, Path(1)).await.unwrap().0;

        assert!(response.contains("Whatever"));
        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_header_returns_board_header(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_board_header(state, Path(1)).await.unwrap().0;

        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_does_that(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db.clone()));

        let request = BoardRename  {
            id: 1,
            name: String::from("New Name Brah")
        };

        let response = post_board_rename(state, Path(1), Form(request)).await.unwrap();
        let board = data::get_board(db, 1).await.unwrap();

        assert_eq!("/boards/1/header", response.location());
        assert_eq!("New Name Brah", board.name);

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_delete_deletes_the_board(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = post_board_delete(state, Path(1)).await.unwrap();

        assert_eq!("/", response.location());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_request_error_if_path_and_form_ref_different_boards(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let request = BoardRename {
            id: 1000,
            name: "This will fail anyway.".to_string(),
        };

        let response = post_board_rename(state, Path(1), Form(request)).await.unwrap_err();

        assert_eq!(response, KanbanError::RequestError("Ambiguous board specified, path says `1` and the form says `1000`".to_string()));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_database_error_if_board_doesnt_exist(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let request = BoardRename {
            id: 1000,
            name: "This will still fail anyway.".to_string(),
        };

        let response = post_board_rename(state, Path(1000), Form(request)).await.unwrap_err();

        assert_eq!(response, KanbanError::DatabaseError("Update failed please try again.".to_string()));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_delete_board_not_found_error_if_board_doesnt_exist(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = post_board_delete(state, Path(1000)).await.unwrap_err();

        assert_eq!(response, KanbanError::BoardNotFound(1000));

        Ok(())
    }

    #[test_case("/")]
    #[test_case("/boards/1")]
    #[test_case("/boards/1/edit")]
    #[test_case("/boards/1/header")]
    #[tokio::test]
    async fn authed_get_pages_redirect_when_anonymous(path: &str) {
        base_auth_get_assertion(path).await;
    }

    #[test_case("/boards")]
    #[test_case("/boards/1/rename")]
    #[test_case("/boards/1/delete")]
    #[tokio::test]
    async fn authed_post_pages_redirect_when_anonymous(path: &str) {
        base_auth_post_assertion(path).await;
    }
}
