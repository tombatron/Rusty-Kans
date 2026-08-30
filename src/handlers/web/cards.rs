use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{create_card_common, delete_card_common, move_card_common, patch_card_common, CreateCardRequest};
use crate::models::{Card, CardMoveEvent, Status};
use crate::state::{ApplicationState, UserDb};
use crate::turbo::TurboStream;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/lists/{list_id}/cards", post(post_card_form))
        .route("/lists/{list_id}/cards/{card_id}/move", post(post_move_card_action))
        .route("/cards/{card_id}/edit", get(get_card_edit).post(patch_card_form))
        .route("/cards/{card_id}/view", get(get_card_by_id))
        .route("/lists/{list_id}/cards/{card_id}/delete", post(delete_card_form))
}

#[derive(Debug, Clone, Template)]
#[template(path = "turbo_move_card.html")]
struct MoveCardTemplate {
    card_id: u64,
    to_list_id: u64,
    card: Card,
}

impl Into<CardMoveEvent> for MoveCardTemplate {
    fn into(self) -> CardMoveEvent {
        CardMoveEvent {
            card_id: self.card_id,
            to_list_id: self.to_list_id,
            card: self.card,
        }
    }
}

async fn post_move_card_action(
    UserDb(db): UserDb,
    State(state): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    move_card_common(db.clone(), list_id, card_id).await?;

    let card = data::get_card(db, card_id).await?;

    let response = MoveCardTemplate {
        card_id,
        to_list_id: list_id,
        card,
    };

    // Discard the potential error response because send will return an error if there are zero
    // active receivers.
    let _ = state.tx.send(response.clone().into());

    Ok(TurboStream(response.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_card.html")]
struct NewCardTemplate {
    id: u64,
    list_id: u64,
    title: String,
    description: Option<String>,
    status: Status,
}

impl From<Card> for NewCardTemplate {
    fn from(value: Card) -> Self {
        NewCardTemplate {
            id: value.id,
            list_id: value.list_id,
            title: value.title,
            description: value.description,
            status: value.status,
        }
    }
}

async fn post_card_form(
    UserDb(db): UserDb,
    Path(list_id): Path<u64>,
    Form(card): Form<CreateCardRequest>,
) -> Result<TurboStream, KanbanError> {
    let card = create_card_common(db, list_id, card).await?;

    let template =  NewCardTemplate::from(card);

    Ok(TurboStream(template.render()?))
}

async fn delete_card_form(
    UserDb(db): UserDb,
    Path((_list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    delete_card_common(db, card_id).await?;

    let response =
        format!("<turbo-stream action=\"remove\" target=\"card-{card_id}\"></turbo-stream>");

    Ok(TurboStream(response))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card_edit.html")]
struct EditCardTemplate {
    id: u64,
    list_id: u64,
    title: String,
    description: Option<String>,
    status: Status,
}

impl From<Card> for EditCardTemplate {
    fn from(value: Card) -> Self {
        EditCardTemplate {
            id: value.id,
            list_id: value.list_id,
            title: value.title,
            description: value.description,
            status: value.status,
        }
    }
}

async fn get_card_edit(
    UserDb(db): UserDb,
    Path(card_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let card = data::get_card(db, card_id).await?;

    let template = EditCardTemplate::from(card);

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card.html")]
struct CardTemplate {
    id: u64,
    list_id: u64,
    title: String,
    description: Option<String>,
    status: Status,
}

impl From<Card> for CardTemplate {
    fn from(value: Card) -> Self {
        CardTemplate {
            id: value.id,
            list_id: value.list_id,
            title: value.title,
            description: value.description,
            status: value.status,
        }
    }
}

async fn patch_card_form(UserDb(db): UserDb, Path(card_id): Path<i64>, Form(card): Form<Card>) -> Result<Redirect, KanbanError> {
    patch_card_common(db, card).await?;

    Ok(Redirect::to(format!("/cards/{card_id}/view").as_str()))
}

async fn get_card_by_id(UserDb(db): UserDb, Path(card_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let card = data::get_card(db, card_id).await?;

    let template = CardTemplate::from(card);

    Ok(Html(template.render()?))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Path, State};
    use sqlx::SqlitePool;
    use crate::data;
    use crate::handlers::tests::get_fake_application_state;
    use crate::handlers::web::cards::*;
    use crate::handlers::web::tests::{base_auth_get_assertion, base_auth_post_assertion};
    use crate::models::Status::Done;
    use test_case::test_case;

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_move_card_action_returns_turbo_directives(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state());
        let db = UserDb(db);

        let response = post_move_card_action(db.clone(), state, Path((2, 1))).await.unwrap().0;

        let card = data::get_card(db.0, 1).await.unwrap();

        assert!(response.contains("card-1"));
        assert!(response.contains("list-cards-2"));
        assert_eq!(2, card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_card_form_adds_new_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = CreateCardRequest {
            title: String::from("Super New Card"),
            description: Some(String::from("Super New Description"))
        };

        let response = post_card_form(db, Path(1), Form(request)).await.unwrap().0;

        assert!(response.contains("card-19"));
        assert!(response.contains("Super New Card"));
        assert!(response.contains("Super New Description"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_card_edit_returns_card_edit_form(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_card_edit(db, Path(1)).await.unwrap().0;

        assert!(response.contains("card-frame-1"));
        assert!(response.contains("Card 1"));
        assert!(response.contains("This is a description"));
        assert!(response.contains("Todo"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn delete_card_form_deletes_the_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = delete_card_form(db.clone(), Path((1, 1))).await.unwrap().0;

        let card = data::get_card(db.0, 1).await;

        assert!(response.contains("card-1"));
        assert!(card.is_err()); // We expect there to be no "card 1".

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn patch_card_form_updates_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let request = Card {
            id: 1,
            list_id: 1,
            title: String::from("Patched Card"),
            description: Some(String::from("Patched Description")),
            status: Done
        };

        let response = patch_card_form(db.clone(), Path(1), Form(request)).await.unwrap();

        let card = data::get_card(db.0, 1).await.unwrap();

        assert_eq!("/cards/1/view", response.location());
        assert_eq!("Patched Card", card.title);
        assert_eq!("Patched Description", card.description.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_card_by_id_returns_card(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_card_by_id(db, Path(1)).await.unwrap().0;

        assert!(response.contains("card-frame-1"));
        assert!(response.contains("This is a description"));

        Ok(())
    }

    #[test_case("/cards/1/edit")]
    #[test_case("/cards/1/view")]
    #[tokio::test]
    async fn authed_get_pages_redirect_when_anonymous(path: &str) {
        base_auth_get_assertion(path).await;
    }

    #[test_case("/lists/1/cards")]
    #[test_case("/lists/1/cards/1/move")]
    #[test_case("/lists/1/cards/1/delete")]
    #[tokio::test]
    async fn authed_post_pages_redirect_when_anonymous(path: &str) {
        base_auth_post_assertion(path).await;
    }
}