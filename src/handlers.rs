use crate::data;
use crate::errors::KanbanError;
use crate::models::{Card, Status};
use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub mod api;
pub mod auth;
pub mod utility;
pub mod web;
pub mod ws;

#[derive(Debug, Serialize, Template)]
#[template(path = "turbo_list_item.html")]
struct ListItemTemplate {
    list_id: u64,
    name: String,
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    name: String,
}

async fn create_list_common(
    db: SqlitePool,
    board_id: u64,
    list_info: CreateListRequest,
) -> Result<ListItemTemplate, KanbanError> {
    let list_id = data::insert_list(db, board_id, &list_info.name).await?;

    Ok(ListItemTemplate {
        list_id,
        name: list_info.name,
    })
}

async fn move_card_common(
    db: SqlitePool,
    list_id: u64,
    card_id: u64,
) -> Result<(), KanbanError> {
    data::update_card_list(db, list_id, card_id).await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct CreateCardRequest {
    title: String,
    description: Option<String>,
}

async fn create_card_common(
    db: SqlitePool,
    list_id: u64,
    card: CreateCardRequest,
) -> Result<Card, KanbanError> {
    let card_id = data::insert_card(db, list_id, &card.title, &card.description).await?;

    Ok(Card {
        id: card_id,
        list_id,
        title: card.title,
        description: card.description,
        status: Status::Todo,
    })
}

async fn delete_card_common(
    db: SqlitePool,
    card_id: u64,
) -> Result<(), KanbanError> {
    data::delete_card(db, card_id).await?;

    Ok(())
}

async fn patch_card_common(
    db: SqlitePool,
    card: Card,
) -> Result<(), KanbanError> {
    let result =
        data::update_card(db, card.id, card.title, card.description, card.status).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError(
            "Update failed. Refresh and try again.".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::models::CardMoveEvent;
    use crate::state::ApplicationState;
    use dashmap::DashMap;
    use oauth2::basic::BasicClient;
    use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
    use sqlx::SqlitePool;

    pub fn get_fake_application_state() -> ApplicationState {
        let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

        let oauth_client = BasicClient::new(ClientId::new("github_client_id".to_string()))
            .set_client_secret(ClientSecret::new("github_client_secret".to_string()))
            .set_auth_uri(
                AuthUrl::new("https://example.com/login/oauth/authorize".to_string()).unwrap(),
            )
            .set_token_uri(
                TokenUrl::new("https://example.com/login/oauth/access_token".to_string()).unwrap(),
            )
            .set_redirect_uri(RedirectUrl::new("http://example.com".to_string()).unwrap());
        
        let db_pools = DashMap::<u64, SqlitePool>::new();

        ApplicationState {
            db_pools,
            tx,
            oauth_client,
        }
    }

    #[sqlx::test(fixtures("boards"))]
    async fn create_list_common_does_the_thing(db: SqlitePool) -> sqlx::Result<()> {
        let request = CreateListRequest {
            name: "Totally new list.".to_string(),
        };

        let response = create_list_common(db, 1, request).await.unwrap();

        assert_eq!(7, response.list_id);
        assert_eq!("Totally new list.", response.name);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn move_card_common_moves_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let card = data::get_card(db.clone(), 1).await.unwrap();

        move_card_common(db.clone(), 2, 1).await.unwrap();

        let moved_card = data::get_card(db, 1).await.unwrap();

        assert_eq!(1, card.list_id);
        assert_eq!(2, moved_card.list_id);

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn create_card_common_creates_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let request = CreateCardRequest {
            title: "This is a new card.".to_string(),
            description: Some("This is a new description.".to_string()),
        };

        let response = create_card_common(db, 1, request).await.unwrap();

        assert!(response.id > 0);
        assert_eq!(1, response.list_id);
        assert_eq!("This is a new card.", response.title);
        assert_eq!("This is a new description.", response.description.unwrap());

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn delete_card_common_deletes_a_card(db: SqlitePool) -> sqlx::Result<()> {
        delete_card_common(db.clone(), 1).await.unwrap();

        let result = data::get_card(db, 1).await.unwrap_err();

        assert!(matches!(result, KanbanError::CardNotFound(_)));

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn patch_card_common_updates_a_card(db: SqlitePool) -> sqlx::Result<()> {
        let request = Card {
            id: 1,
            list_id: 1,
            title: "This is an updated title.".to_string(),
            description: Some("This is an updated description.".to_string()),
            status: Status::Done,
        };

        patch_card_common(db.clone(), request).await.unwrap();

        let updated_card = data::get_card(db, 1).await.unwrap();

        assert_eq!(1, updated_card.id);
        assert_eq!(1, updated_card.list_id);
        assert_eq!("This is an updated title.", updated_card.title);
        assert_eq!(
            "This is an updated description.",
            updated_card.description.unwrap()
        );
        assert!(matches!(updated_card.status, Status::Done));

        Ok(())
    }

    #[sqlx::test(fixtures("boards"))]
    async fn patch_card_common_returns_err_if_card_not_updated(db: SqlitePool) -> sqlx::Result<()> {
        let request = Card {
            id: 1000,
            list_id: 1,
            title: "Whatever".to_string(),
            description: None,
            status: Status::Doing,
        };

        let response = patch_card_common(db, request).await.unwrap_err();

        assert!(matches!(response, KanbanError::DatabaseError(_)));

        Ok(())
    }
}
