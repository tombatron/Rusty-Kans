use crate::data;
use crate::errors::KanbanError;
use crate::handlers::web::NewContainerFormTemplate;
use crate::handlers::{CreateListRequest, create_list_common};
use crate::models::List;
use crate::state::{ApplicationState, UserDb};
use crate::turbo::TurboStream;
use crate::validation::FormErrors;
use askama::Template;
use axum::extract::Path;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use axum::http::StatusCode;
use garde::Validate;
use serde::Deserialize;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards/{board_id}/lists", post(post_list_form))
        .route("/lists/{list_id}/edit", get(get_list_edit))
        .route("/lists/{list_id}/rename", post(post_list_rename))
        .route("/lists/{list_id}/header", get(get_list_header))
        .route("/lists/{list_id}/delete", post(post_list_delete))
}

#[derive(Debug, Template)]
#[template(
    source = "{% import \"macros.html\" as macros %}{{ macros::add_new_container_form(context) }}",
    ext = "html"
)]
struct NewListErrorTemplate {
    context: NewContainerFormTemplate<CreateListRequest>,
}

async fn post_list_form(
    UserDb(db): UserDb,
    Path(board_id): Path<u64>,
    Form(list_info): Form<CreateListRequest>,
) -> Result<Response, KanbanError> {
    let validation_errors = list_info.validate();

    if let Err(errors) = validation_errors {
        let validation_response = NewListErrorTemplate {
            context: NewContainerFormTemplate {
                sub_id: "list".to_string(),
                action: format!("/boards/{board_id}/lists"),
                place_holder: "".to_string(),
                button_sub_label: "List".to_string(),
                errors: Some(FormErrors::from_report(&errors)),
                request: Some(list_info),
            },
        };

        return Ok((StatusCode::UNPROCESSABLE_ENTITY, TurboStream(validation_response.render()?)).into_response());
    }

    let created_list = create_list_common(db, board_id, list_info).await?;

    Ok(TurboStream(created_list.render()?).into_response())
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title.html")]
struct ListHeader {
    list: List,
}

async fn get_list_header(
    UserDb(db): UserDb,
    Path(list_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let list = data::get_list_header(db, list_id).await?;

    let template = ListHeader { list };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title_edit.html")]
struct ListHeaderEdit {
    list: List,
    errors: Option<FormErrors>
}

async fn get_list_edit(
    UserDb(db): UserDb,
    Path(list_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let list = data::get_list_header(db, list_id).await?;

    let template = ListHeaderEdit { list, errors: None };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize, Validate)]
struct ListRename {
    #[garde(length(min = 1, max = 100))]
    name: String,
}

async fn post_list_rename(
    UserDb(db): UserDb,
    Path(list_id): Path<u64>,
    Form(list_header): Form<ListRename>,
) -> Result<Response, KanbanError> {
    let validation_errors = list_header.validate();

    if let Err(errors) = validation_errors {
        let validation_response = ListHeaderEdit {
            list: List { id: list_id, board_id: 0, name: list_header.name, cards: vec!() },
            errors: Some(FormErrors::from_report(&errors)),
        };

        return Ok((StatusCode::UNPROCESSABLE_ENTITY, TurboStream(validation_response.render()?)).into_response());
    }

    let result = data::update_list(db, list_id, list_header.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError(
            "Update failed please try again.".to_string(),
        ));
    }

    Ok(Redirect::to(format!("/lists/{list_id}/header").as_str()).into_response())
}

async fn post_list_delete(
    UserDb(db): UserDb,
    Path(list_id): Path<u64>,
) -> Result<TurboStream, KanbanError> {
    data::delete_list(db, list_id).await?;

    let result =
        format!("<turbo-stream action=\"remove\" target=\"list-{list_id}\"></turbo-stream>");

    Ok(TurboStream(result))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::handlers::web::tests::{base_auth_get_assertion, base_auth_post_assertion, get_response_body};
    use sqlx::SqlitePool;
    use test_case::test_case;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_form_adds_new_list(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateListRequest {
            name: "This is a new list".to_string(),
        };

        let response = post_list_form(db, Path(1), Form(request)).await.unwrap();
        let response = get_response_body(response).await;

        assert!(response.contains("list-7"));
        assert!(response.contains("<h3 class=\"list-title\">This is a new list</h3>"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_form_handles_validation_errors(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateListRequest { name: "".to_string(), };

        let response = post_list_form(db, Path(1), Form(request)).await.unwrap();
        let response_status = response.status();
        let response = get_response_body(response).await;

        assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, response_status);
        assert!(response.contains("length is lower than 1"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_list_header_returns_list_header(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_list_header(db, Path(1)).await.unwrap().0;

        assert!(response.contains("list-title-1"));
        assert!(response.contains("<h3 class=\"list-title\">First List</h3>"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_list_edit_returns_edit_html(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_list_edit(db, Path(1)).await.unwrap().0;

        assert!(response.contains("<input type=\"hidden\" name=\"id\" value=\"1\">"));
        assert!(response.contains("<input type=\"text\" name=\"name\" value=\"First List\">"));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_rename_renames_list(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = ListRename {
            name: "Renamed list".to_string(),
        };

        let response = post_list_rename(db.clone(), Path(1), Form(request))
            .await
            .unwrap();

        let redirect_location = response.headers().get("location").unwrap().to_str().unwrap();

        let renamed_list = data::get_list_header(db.0, 1).await.unwrap();

        assert_eq!("/lists/1/header", redirect_location);
        assert_eq!("Renamed list", renamed_list.name);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_rename_handles_validation_failure(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = ListRename {
            name: "".to_string(),
        };

        let response = post_list_rename(db.clone(), Path(1), Form(request)).await.unwrap();
        let response_status = response.status();
        let response_body = get_response_body(response).await;

        let renamed_list = data::get_list_header(db.0, 1).await.unwrap();

        assert!(response_body.contains("length is lower than 1"));
        assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, response_status);
        assert_eq!("First List", renamed_list.name); // Shouldn't have changed.

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_rename_fails_when_delete_affects_nothing(
        db: SqlitePool,
    ) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = ListRename {
            name: "Renamed list".to_string(),
        };

        let response = post_list_rename(db, Path(1000), Form(request))
            .await
            .unwrap_err();

        assert!(matches!(response, KanbanError::DatabaseError(_)));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn post_list_delete_deletes_the_list(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = post_list_delete(db, Path(1)).await.unwrap().0;

        assert!(
            response.contains("<turbo-stream action=\"remove\" target=\"list-1\"></turbo-stream>")
        );

        Ok(())
    }

    #[test_case("/lists/1/edit")]
    #[test_case("/lists/1/header")]
    #[tokio::test]
    async fn authed_get_pages_redirect_when_anonymous(path: &str) {
        base_auth_get_assertion(path).await;
    }

    #[test_case("/boards/1/lists")]
    #[test_case("/lists/1/rename")]
    #[test_case("/lists/1/delete")]
    #[tokio::test]
    async fn authed_post_pages_redirect_when_anonymous(path: &str) {
        base_auth_post_assertion(path).await;
    }
}
