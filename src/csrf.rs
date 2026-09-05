use crate::errors::KanbanError;
use tower_sessions::Session;

const CSRF_SECRET_KEY: &str = "CSRF_SECRET";

pub fn generate_secret() -> [u8; 32] {
    let mut secret_array= [0u8; 32];

    rand::fill(&mut secret_array);

    secret_array
}

pub fn mask(secret: &[u8; 32]) -> String {
    let mut secret_mask = [0u8; 32];
    rand::fill(& mut secret_mask);

    let mut secret_masked = [0u8; 32];

    for i in 0..32 {
        let mask_value = secret_mask[i];
        let secret_value = secret[i];

        secret_masked[i] = mask_value ^ secret_value;
    }

    format!("{}:{}", hex::encode(secret_mask), hex::encode(secret_masked))
}

pub fn verify(token: &str, secret: &[u8; 32]) -> bool {
    let token_parts: Vec<&str> = token.split(":").collect();

    if token_parts.len() != 2 || token_parts[0].len() != 64 || token_parts[1].len() != 64 {
        return false;
    }

    let secret_mask = hex::decode(token_parts[0]).unwrap_or([0u8; 32].to_vec());
    let secret_masked = hex::decode(token_parts[1]).unwrap_or([0u8; 32].to_vec());

    let mut reconciled_mask = [0u8; 32];

    for i in 0..32 {
        reconciled_mask[i] = secret_masked[i] ^ secret[i];
    }

    let mut result = true;

    for i in 0..32 {
        result = reconciled_mask[i] == secret_mask[i] && result;
    }

    result
}

pub async fn get_or_create_secret(session: &Session) -> Result<[u8; 32], KanbanError> {
    let stored_secret: Option<[u8; 32]> = session.get(CSRF_SECRET_KEY).await?;

    if let Some(secret) = stored_secret {
        return Ok(secret);
    }

    let new_secret = generate_secret();

    session.insert(CSRF_SECRET_KEY, new_secret).await?;

    Ok(new_secret)
}

#[cfg(test)]
pub mod tests {
    use crate::csrf::{generate_secret, get_or_create_secret, mask, verify, CSRF_SECRET_KEY};
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    #[test]
    fn generate_secret_generates_random_secret() {
        let secret = generate_secret();

        assert_eq!(32, secret.len());
    }

    #[test]
    fn mask_will_mask_a_random_value_with_provided_secret() {
        let secret = generate_secret();

        let masked_secret_value = mask(&secret);

        assert_eq!(129, masked_secret_value.len());
    }

    #[test]
    fn verify_can_verify_valid_token() {
        let secret = generate_secret();
        let token = mask(&secret);

        let verify_result = verify(&token, &secret);

        assert!(verify_result);
    }

    #[test]
    fn verify_can_invalidate_bad_token() {
        let test_token = mask(&generate_secret());

        // Generated a new secret which should cause validation to fail.
        let invalid_result = verify(&test_token, &generate_secret());

        assert!(!invalid_result);
    }

    #[test]
    fn verify_can_invalidate_malformed_token() {
        let test_token = "not-a-valid-token-at-all".to_string();

        let invalid_result = verify(&test_token, &generate_secret());

        assert!(!invalid_result);
    }

    #[test]
    fn verify_can_invalidate_well_formed_but_invalid_token() {
        let test_token = "00:00".to_string();

        let invalid_result = verify(&test_token, &generate_secret());

        assert!(!invalid_result);
    }

    #[tokio::test]
    async fn get_or_create_secret_will_create_a_secret_on_the_first_call() {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);

        let new_secret = get_or_create_secret(&session).await.unwrap();

        let stored_secret: Option<[u8; 32]> = session.get(CSRF_SECRET_KEY).await.unwrap();

        assert!(stored_secret.is_some());
        assert_eq!(stored_secret, Some(new_secret));
    }

    #[tokio::test]
    async fn get_or_create_secret_will_return_existing_secret_on_second_call() {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);

        // First call should generate a new secret.
        let new_secret = get_or_create_secret(&session).await.unwrap();

        // Second call should yield the same secret because of the session storage.
        let same_secret = get_or_create_secret(&session).await.unwrap();

        assert_eq!(new_secret, same_secret);
    }
}