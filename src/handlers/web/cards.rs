use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{create_card_common, delete_card_common, move_card_common, patch_card_common, CreateCardRequest};
use crate::models::{Card, CardMoveEvent};
use crate::state::ApplicationState;
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
    State(state): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    move_card_common(State(state.clone()), list_id, card_id).await?;

    let card = data::get_card(state.db, card_id).await?;

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
    card: Card,
}

async fn post_card_form(
    State(state): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Form(card): Form<CreateCardRequest>,
) -> Result<TurboStream, KanbanError> {
    let card = create_card_common(State(state), list_id, card).await?;

    let template =  NewCardTemplate { card };

    Ok(TurboStream(template.render()?))
}

async fn delete_card_form(
    State(state): State<ApplicationState>,
    Path((_list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    delete_card_common(State(state), card_id).await?;

    let response =
        format!("<turbo-stream action=\"remove\" target=\"card-{card_id}\"></turbo-stream>");

    Ok(TurboStream(response))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card_edit.html")]
struct EditCardTemplate {
    card: Card,
}

async fn get_card_edit(
    State(state): State<ApplicationState>,
    Path(card_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let card = data::get_card(state.db, card_id).await?;

    let template = EditCardTemplate { card };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card.html")]
struct CardTemplate {
    card: Card,
}

async fn patch_card_form(State(state): State<ApplicationState>, Path(card_id): Path<i64>, Form(card): Form<Card>) -> Result<Redirect, KanbanError> {
    patch_card_common(State(state), card).await?;

    Ok(Redirect::to(format!("/cards/{card_id}/view").as_str()))
}

async fn get_card_by_id(State(state): State<ApplicationState>, Path(card_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let card = data::get_card(state.db, card_id).await?;

    let template = CardTemplate { card};

    Ok(Html(template.render()?))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Path, State};
    use sqlx::SqlitePool;
    use crate::data;
    use crate::handlers::tests::get_fake_application_state;
    use crate::handlers::web::cards::*;
    use crate::models::Status::Done;

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_move_card_action_returns_turbo_directives(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db.clone()));

        let response = post_move_card_action(state, Path((2, 1))).await.unwrap().0;

        let card = data::get_card(db, 1).await.unwrap();

        assert!(response.contains("card-1"));
        assert!(response.contains("list-cards-2"));
        assert_eq!(2, card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn post_card_form_adds_new_card(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let request = CreateCardRequest {
            title: String::from("Super New Card"),
            description: Some(String::from("Super New Description"))
        };

        let response = post_card_form(state, Path(1), Form(request)).await.unwrap().0;

        assert!(response.contains("card-19"));
        assert!(response.contains("Super New Card"));
        assert!(response.contains("Super New Description"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_card_edit_returns_card_edit_form(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_card_edit(state, Path(1)).await.unwrap().0;

        assert!(response.contains("card-frame-1"));
        assert!(response.contains("Card 1"));
        assert!(response.contains("This is a description"));
        assert!(response.contains("Todo"));

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn delete_card_form_deletes_the_card(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db.clone()));

        let response = delete_card_form(state, Path((1, 1))).await.unwrap().0;

        let card = data::get_card(db, 1).await;

        assert!(response.contains("card-1"));
        assert!(card.is_err()); // We expect there to be no "card 1".

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn patch_card_form_updates_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db.clone()));

        let request = Card {
            id: 1,
            list_id: 1,
            title: String::from("Patched Card"),
            description: Some(String::from("Patched Description")),
            status: Done
        };

        let response = patch_card_form(state, Path(1), Form(request)).await.unwrap();

        let card = data::get_card(db, 1).await.unwrap();

        assert_eq!("/cards/1/view", response.location());
        assert_eq!("Patched Card", card.title);
        assert_eq!("Patched Description", card.description.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures(path="../../fixtures", scripts("boards")))]
    async fn get_card_by_id_returns_card(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_card_by_id(state, Path(1)).await.unwrap().0;

        assert!(response.contains("card-frame-1"));
        assert!(response.contains("This is a description"));

        Ok(())
    }
}