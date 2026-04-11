use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::info;

/// Calculates SHA-256 hash of content.
pub fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub enum SyncStatus {
    Ingested,
    Skipped,
    Updated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash_empty_input() {
        let input = b"";
        let hash = calculate_hash(input);
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_calculate_hash_known_input() {
        let input = b"hello world";
        let hash = calculate_hash(input);
        // SHA256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_hash_deterministic() {
        let input = b"test content";
        let hash1 = calculate_hash(input);
        let hash2 = calculate_hash(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_hash_different_inputs_different_hashes() {
        let hash1 = calculate_hash(b"input1");
        let hash2 = calculate_hash(b"input2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sync_status_variants() {
        // Test that SyncStatus variants can be created and used
        let _ = SyncStatus::Ingested;
        let _ = SyncStatus::Skipped;
        let _ = SyncStatus::Updated;
    }
}

/// Checks if a translation needs syncing based on its hash.
pub async fn check_and_sync_translation(
    pool: &PgPool,
    translation_id: &str,
    file_hash: &str,
) -> Result<SyncStatus, sqlx::Error> {
    let existing =
        sqlx::query_scalar::<_, Option<String>>("SELECT json_hash FROM translations WHERE id = $1")
            .bind(translation_id)
            .fetch_optional(pool)
            .await?;

    match existing {
        Some(Some(stored_hash)) if stored_hash == file_hash => {
            info!("Translation {} has no changes, skipping", translation_id);
            Ok(SyncStatus::Skipped)
        }
        Some(_) => {
            info!("Translation {} has changed, re-ingesting", translation_id);
            Ok(SyncStatus::Updated)
        }
        None => {
            info!("Translation {} is new, ingesting", translation_id);
            Ok(SyncStatus::Ingested)
        }
    }
}
