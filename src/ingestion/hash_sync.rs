use sha2::{Sha256, Digest};
use sqlx::PgPool;
use tracing::info;

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

pub async fn check_and_sync_translation(
    pool: &PgPool,
    translation_id: &str,
    file_hash: &str,
) -> Result<SyncStatus, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, Option<String>>(
        "SELECT json_hash FROM translations WHERE id = $1",
    )
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
