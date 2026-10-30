use std::time::Duration;

use crate::data;
use crate::errors::KanbanError;
use crate::handlers::auth::AUTHENTICATED_USER_KEY;
use crate::handlers::web::NewContainerFormTemplate;
use crate::models::security::LoggedInUser;
use crate::models::{Board, BoardAccessGrant, List, Permission};
use crate::state::{ApplicationState, CsrfTokenValue, UserDb, get_database_id};
use crate::turbo::TurboStream;
use crate::validation::FormErrors;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use garde::Validate;
use serde::Deserialize;
use serde_json::Value;
use sqlx::types::chrono::Utc;
use tower_sessions::Session;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))
        .route("/boards/{board_id}/edit", get(get_board_edit))
        .route("/boards/{board_id}/header", get(get_board_header))
        .route("/boards/{board_id}/rename", post(post_board_rename))
        .route("/boards/{board_id}/delete", post(post_board_delete))
        .route("/boards/{board_id}/share", get(get_board_share).post(post_board_share))
        .route("/redeem/{owner_database_id}/{share_code}", get(get_redeem_code))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateBoardRequest {
    #[garde(length(min = 1, max = 100))]
    name: String,
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_board.html")]
struct NewBoardTemplate {
    id: u64,
    name: String,
    csrf_token: String,
}

#[derive(Debug, Template)]
#[template(
    source = "{% import \"macros.html\" as macros %}{{ macros::add_new_container_form(context, csrf_token) }}",
    ext = "html"
)]
struct NewBoardErrorTemplate {
    context: NewContainerFormTemplate<CreateBoardRequest>,
    csrf_token: String,
}

async fn post_board_form(
    CsrfTokenValue(csrf_token): CsrfTokenValue,
    UserDb(db): UserDb,
    Form(board): Form<CreateBoardRequest>,
) -> Result<Response, KanbanError> {
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
            },
            csrf_token,
        };

        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            TurboStream(validation_response.render()?),
        )
            .into_response());
    }

    let board_id = data::insert_board(db, &board.name).await?;

    let template = NewBoardTemplate {
        id: board_id as u64,
        name: board.name,
        csrf_token,
    };

    Ok(TurboStream(template.render()?).into_response())
}

#[derive(Debug, Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    board: Board,
    list_ids: Vec<i64>,
    new_list: NewContainerFormTemplate<List>,
    csrf_token: String,
}

pub async fn get_board(
    CsrfTokenValue(csrf_token): CsrfTokenValue,
    UserDb(db): UserDb,
    Path(board_id): Path<i64>,
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
        list_ids: board.list_ids,
        new_list,
        csrf_token,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title_edit.html")]
struct BoardHeaderEdit {
    board: Board,
    errors: Option<FormErrors>,
    csrf_token: String,
}

async fn get_board_edit(
    CsrfTokenValue(csrf_token): CsrfTokenValue,
    UserDb(db): UserDb,
    Path(board_id): Path<i64>,
) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(db, board_id).await?;

    let template = BoardHeaderEdit {
        board,
        errors: None,
        csrf_token,
    };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title.html")]
struct BoardHeader {
    board: Board,
}

async fn get_board_header(
    UserDb(db): UserDb,
    Path(board_id): Path<i64>,
) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(db, board_id).await?;

    let template = BoardHeader { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize, Validate)]
struct BoardRename {
    #[garde(skip)]
    id: i64,
    #[garde(length(min = 1, max = 100))]
    name: String,
}

async fn post_board_rename(
    CsrfTokenValue(csrf_token): CsrfTokenValue,
    UserDb(db): UserDb,
    Path(board_id): Path<i64>,
    Form(board): Form<BoardRename>,
) -> Result<Response, KanbanError> {
    if board_id != board.id {
        return Err(KanbanError::RequestError(format!(
            "Ambiguous board specified, path says `{}` and the form says `{}`",
            board_id, board.id
        )));
    }

    let validation_errors = board.validate();

    if let Err(errors) = validation_errors {
        let validation_response = BoardHeaderEdit {
            board: Board {
                id: board_id,
                name: board.name,
            },
            errors: Some(FormErrors::from_report(&errors)),
            csrf_token,
        };

        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            TurboStream(validation_response.render()?),
        )
            .into_response());
    }

    let result = data::update_board(db, board_id, board.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError(
            "Update failed please try again.".to_string(),
        ));
    }

    Ok(Redirect::to(format!("/boards/{}/header", board_id).as_str()).into_response())
}

async fn post_board_delete(
    UserDb(db): UserDb,
    Path(board_id): Path<i64>,
) -> Result<Redirect, KanbanError> {
    let result = data::delete_board(db, board_id).await?;

    if result != 1 {
        return Err(KanbanError::BoardNotFound(board_id));
    }

    Ok(Redirect::to("/"))
}

#[derive(Debug, Clone, Deserialize)]
struct SharePermission {
    permission: Permission,
    expire_hours: u64
}

#[derive(Debug, Template)]
#[template(path = "share_board.html")]
struct ShareBoard {
    base_url: String,
    owner_database_id: String,
    share_code: String,
    permission: Permission,
}

async fn post_board_share(
    State(state): State<ApplicationState>,
    session: Session,
    UserDb(db): UserDb, 
    Path(board_id): Path<i64>, 
    Form(share): Form<SharePermission>) -> Result<Response, KanbanError> {
        
    let mut share_code_array = [0u8; 32];

    rand::fill(&mut share_code_array);

    let share_code = hex::encode(share_code_array);

    let expires_at = Utc::now() + Duration::from_hours(share.expire_hours);

    let _ = data::add_share_link(db, board_id, share.permission.clone(), expires_at, share_code.clone()).await?;

    // Unwrap here should be safe since this endpoint requires the user to be logged in anyway.
    let current_user = session.get::<LoggedInUser>(AUTHENTICATED_USER_KEY).await?.unwrap();

    let owner_database_id = get_database_id(current_user.id, current_user.source);

    let template = ShareBoard {
        base_url: state.base_url,
        share_code, 
        permission: share.permission,
        owner_database_id,
    };

    Ok(TurboStream(template.render()?).into_response())
}

#[derive(Debug, Template)]
#[template(path = "share_board_form.html")]
struct ShareBoardForm {
    board_id: i64,
    csrf_token: String,

}

async fn get_board_share(CsrfTokenValue(csrf_token): CsrfTokenValue, Path(board_id): Path<i64>) -> Result<Html<String>, KanbanError> {
    let template = ShareBoardForm {
        board_id,
        csrf_token,
    };

    let response = template.render()?;

    Ok(Html(response))
}

async fn get_redeem_code(State(state): State<ApplicationState>, UserDb(db): UserDb, session: Session, Path((owner_database_id, share_code)): Path<(String, String)>) -> Result<Redirect, KanbanError> {
    let current_user = session.get::<LoggedInUser>(AUTHENTICATED_USER_KEY).await?.unwrap();

    // Redeem the code here. This will shoot out to the other backend endpoint and redeem the share code. 
    let client = reqwest::Client::new();

    let res: Value  = client.post(format!("/backend/{owner_database_id}/redeem"))
        .header("X-backend-key", state.secure_key)
        .form(&[
            ("share_code", share_code),
            ("display_name", current_user.name),
            ("user_id", current_user.id.to_string()),
            ("source", current_user.source),
            ("avatar_url", "".to_string()), // Just have this here for now, it's required but not used yet.
        ])
        .send()
        .await?
        .json()
        .await?;

    // We need to write board_access_grants
    let access_grant = BoardAccessGrant {
        owner_database_id: res.get("owner_database_id").unwrap().to_string(),
        remote_board_id: res.get("remote_board_id").unwrap().as_i64().unwrap(),
        remote_board_name: res.get("remote_board_name").unwrap().to_string(),
        created_at: Utc::now()
    };

    let _ = data::add_board_access_grant(db, access_grant).await?;

    // For now we'll redirect to the root, but later we'll go directly to the shared board.
    Ok(Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::handlers::tests::get_fake_application_state;
    use crate::handlers::web::boards::*;
    use crate::handlers::web::tests::*;
    use axum::Form;
    use axum::extract::Path;
    use sqlx::SqlitePool;
    use test_case::test_case;
    use tower_sessions::MemoryStore;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_form_creates_a_new_board(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateBoardRequest {
            name: String::from("New Board Dude."),
        };

        let response = get_response_body(
            post_board_form(CsrfTokenValue("token".to_string()), db, Form(request))
                .await
                .unwrap(),
        )
        .await;

        assert!(response.contains("New Board Dude."));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_form_handles_failed_validation(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateBoardRequest {
            name: String::from(""),
        };

        let response = post_board_form(CsrfTokenValue("token".to_string()), db, Form(request))
            .await
            .unwrap();

        let response_status = response.status();
        let response_body = get_response_body(response).await;

        assert!(response_body.contains("length is lower than 1"));
        assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, response_status);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_board_returns_a_board_page(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board(CsrfTokenValue("token".to_string()), db, Path(2))
            .await
            .unwrap()
            .0;

        assert!(response.contains("Another Board"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_board_edit_returns_edit_form(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board_edit(CsrfTokenValue("token".to_string()), db, Path(1))
            .await
            .unwrap()
            .0;

        assert!(response.contains("Whatever"));
        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_board_header_returns_board_header(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board_header(db, Path(1)).await.unwrap().0;

        assert!(response.contains("board-title-1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_rename_handles_failed_validation(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1,
            name: String::from(""),
        };

        let response = post_board_rename(
            CsrfTokenValue("token".to_string()),
            db.clone(),
            Path(1),
            Form(request),
        )
        .await
        .unwrap();
        let response_status = response.status();
        let response_body = get_response_body(response).await;

        let board = data::get_board(db.0, 1).await.unwrap();

        assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, response_status);
        assert_eq!("Whatever", board.name);
        assert!(response_body.contains("length is lower than 1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_rename_does_that(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1,
            name: String::from("New Name Brah"),
        };

        let response = post_board_rename(
            CsrfTokenValue("token".to_string()),
            db.clone(),
            Path(1),
            Form(request),
        )
        .await
        .unwrap();
        let response = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        let board = data::get_board(db.0, 1).await.unwrap();

        assert_eq!("/boards/1/header", response);
        assert_eq!("New Name Brah", board.name);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_delete_deletes_the_board(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = post_board_delete(db, Path(1)).await.unwrap();

        assert_eq!("/", response.location());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_rename_request_error_if_path_and_form_ref_different_boards(
        db: SqlitePool,
    ) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1000,
            name: "This will fail anyway.".to_string(),
        };

        let response = post_board_rename(
            CsrfTokenValue("token".to_string()),
            db,
            Path(1),
            Form(request),
        )
        .await
        .unwrap_err();

        assert_eq!(
            response,
            KanbanError::RequestError(
                "Ambiguous board specified, path says `1` and the form says `1000`".to_string()
            )
        );

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_rename_database_error_if_board_doesnt_exist(
        db: SqlitePool,
    ) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = BoardRename {
            id: 1000,
            name: "This will still fail anyway.".to_string(),
        };

        let response = post_board_rename(
            CsrfTokenValue("token".to_string()),
            db,
            Path(1000),
            Form(request),
        )
        .await
        .unwrap_err();

        assert_eq!(
            response,
            KanbanError::DatabaseError("Update failed please try again.".to_string())
        );

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_delete_board_not_found_error_if_board_doesnt_exist(
        db: SqlitePool,
    ) -> sqlx::Result<()> {
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

    #[test_case("/boards")]
    #[test_case("/boards/1/rename")]
    #[test_case("/boards/1/delete")]
    #[tokio::test]
    async fn missing_csrf_token_on_post_is_rejected(path: &str) {
        base_csrf_rejection_assertion(path).await;
    }

    #[test_case("/boards", vec!(("name".to_string(), "test".to_string())))]
    #[tokio::test]
    async fn present_csrf_token_on_post_is_accepted(path: &str, form: Vec<(String, String)>) {
        base_csrf_acceptance_assertion(path, form).await;
    }

    #[test_case("/boards/1/rename", "/boards/1/header", vec!(("id".to_string(), "1".to_string()), ("name".to_string(), "new_name".to_string())))]
    #[test_case("/boards/2/delete", "/", vec!())]
    #[tokio::test]
    async fn present_csrf_token_on_post_is_accepted_and_redirected(path: &str, redirect_url: &str, form: Vec<(String, String)>){
        base_csrf_acceptance_redirect_assertion(path, redirect_url, form).await;
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_board_share_returns_share_code(db: SqlitePool) -> sqlx::Result<()> {
        // Setup a fake user in a fake session. 
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);

        let logged_in_user = LoggedInUser {
            id: -1000,
            name: "Your face".to_string(),
            source: "whatever".to_string(),
        };

        session.insert(AUTHENTICATED_USER_KEY, logged_in_user).await.unwrap();


        let db = UserDb(db);

        let request = SharePermission {
            permission: Permission::Edit,
            expire_hours: 10,
        };

        let fake_application_state = get_fake_application_state();

        let response_one = post_board_share(State(fake_application_state.clone()), session.clone(), db.clone(), Path(1), Form(request.clone())).await.unwrap();
        let response_two = post_board_share(State(fake_application_state), session, db, Path(1), Form(request)).await.unwrap();

        let body_one = get_response_body(response_one).await;
        let body_two = get_response_body(response_two).await;

        assert_ne!(body_one, body_two);

        Ok(())
    }
}
