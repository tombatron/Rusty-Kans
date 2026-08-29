use crate::data;
use crate::errors::KanbanError;
use crate::models::{Board, Card, List};
use crate::state::{ApplicationState, UserDb};
use crate::turbo::TurboStream;
use askama::Template;
use axum::extract::Path;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use garde::Validate;
use crate::handlers::web::NewContainerFormTemplate;
use crate::validation::FormErrors;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))
        .route("/boards/{board_id}/edit", get(get_board_edit))
        .route("/boards/{board_id}/header", get(get_board_header))
        .route("/boards/{board_id}/rename", post(post_board_rename))
        .route("/boards/{board_id}/delete", post(post_board_delete))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateBoardRequest {
    #[garde(length(min = 2, max = 100))]
    name: String,
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_board.html")]
struct NewBoardTemplate {
    id: u64,
    name: String
}

#[derive(Debug, Template)]
#[template(source = "{% import \"macros.html\" as macros %}{{ macros::add_new_container_form(context) }}", ext= "html")]
struct NewBoardErrorTemplate {
    context: NewContainerFormTemplate<CreateBoardRequest>,
}

async fn post_board_form(UserDb(db): UserDb, Form(board): Form<CreateBoardRequest>) -> Result<Response, KanbanError> {
    let validation_errors = board.validate();

    if let Err(errors) = validation_errors {
        let validation_response = NewBoardErrorTemplate {
            context: NewContainerFormTemplate {
                sub_id: "board".to_string(),
                action: "/boards".to_string(),
                place_holder: "".to_string(),
                button_sub_label: "Board".to_string(),
                errors: Some(FormErrors::from_report(&errors)),
                request: Some(board),
            }
        };

        return Ok((StatusCode::UNPROCESSABLE_ENTITY, TurboStream(validation_response.render()?)).into_response())
    }

    let board_id = data::insert_board(db, &board.name).await?;

    let template = NewBoardTemplate {
        id: board_id as u64,
        name: board.name,
    };

    Ok(TurboStream(template.render()?).into_response())
}

#[derive(Debug, Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    board: Board,
    lists: Vec<List>,
    new_list: NewContainerFormTemplate<List>
}

pub async fn get_board(
    UserDb(db): UserDb,
    Path(board_id): Path<u64>
) -> Result<Html<String>, KanbanError> {
    let board = data::get_board_with_lists(db, board_id).await?;

    let new_list = NewContainerFormTemplate {
        sub_id: "list".to_string(),
        action: format!("/boards/{board_id}/lists"),
        place_holder: "New list&hellip;".to_string(),
        button_sub_label: "List".to_string(),
        errors: None,
        request: None,
    };

    let response_template = BoardTemplate {
        board: board.board,
        lists: board.lists,
        new_list,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title_edit.html")]
struct BoardHeaderEdit {
    board: Board
}

async fn get_board_edit(UserDb(db): UserDb, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(db, board_id).await?;

    let template = BoardHeaderEdit { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title.html")]
struct BoardHeader {
    board: Board
}

async fn get_board_header(UserDb(db): UserDb, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(db, board_id).await?;

    let template = BoardHeader { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
struct BoardRename {
    id: u64,
    name: String,
}

async fn post_board_rename(UserDb(db): UserDb, Path(board_id):Path<u64>, Form(board): Form<BoardRename>) -> Result<Redirect, KanbanError> {
    if board_id != board.id {
        return Err(KanbanError::RequestError(format!("Ambiguous board specified, path says `{}` and the form says `{}`", board_id, board.id)));
    }

    let result = data::update_board(db, board_id, board.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/boards/{}/header", board_id).as_str()))
}

async fn post_board_delete(UserDb(db): UserDb, Path(board_id):Path<u64>) -> Result<Redirect, KanbanError> {
    let result = data::delete_board(db, board_id).await?;

    if result != 1 {
        return Err(KanbanError::BoardNotFound(board_id));
    }

    Ok(Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use crate::handlers::web::boards::*;
    use crate::handlers::web::tests::*;
    use axum::Form;
    use axum::extract::Path;
    use sqlx::SqlitePool;
    use test_case::test_case;

    async fn get_response_body(response: Response) -> String {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_form_creates_a_new_board(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateBoardRequest {
            name: String::from("New Board Dude.")
        };

        let response = get_response_body(post_board_form(db, Form(request)).await.unwrap()).await;

        assert!(response.contains("New Board Dude."));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_returns_a_board_page(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board(db, Path(2)).await.unwrap().0;

        assert!(response.contains("Another Board"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_edit_returns_edit_form(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board_edit(db, Path(1)).await.unwrap().0;

        assert!(response.contains("Whatever"));
        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_board_header_returns_board_header(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board_header(db, Path(1)).await.unwrap().0;

        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_does_that(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename  {
            id: 1,
            name: String::from("New Name Brah")
        };

        let response = post_board_rename(db.clone(), Path(1), Form(request)).await.unwrap();
        let board = data::get_board(db.0, 1).await.unwrap();

        assert_eq!("/boards/1/header", response.location());
        assert_eq!("New Name Brah", board.name);

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_delete_deletes_the_board(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = post_board_delete(db, Path(1)).await.unwrap();

        assert_eq!("/", response.location());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_request_error_if_path_and_form_ref_different_boards(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1000,
            name: "This will fail anyway.".to_string(),
        };

        let response = post_board_rename(db, Path(1), Form(request)).await.unwrap_err();

        assert_eq!(response, KanbanError::RequestError("Ambiguous board specified, path says `1` and the form says `1000`".to_string()));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_rename_database_error_if_board_doesnt_exist(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1000,
            name: "This will still fail anyway.".to_string(),
        };

        let response = post_board_rename(db, Path(1000), Form(request)).await.unwrap_err();

        assert_eq!(response, KanbanError::DatabaseError("Update failed please try again.".to_string()));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_board_delete_board_not_found_error_if_board_doesnt_exist(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = post_board_delete(db, Path(1000)).await.unwrap_err();

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
