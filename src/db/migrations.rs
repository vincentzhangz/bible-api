use sqlx::PgPool;
use tracing::info;

/// Migrations are applied once and recorded in `schema_migrations`.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial",
        include_str!("../../migrations/001_initial.sql"),
    ),
    (
        "002_books_slug",
        include_str!("../../migrations/002_books_slug.sql"),
    ),
    (
        "003_canonical_books",
        include_str!("../../migrations/003_canonical_books.sql"),
    ),
];

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\
             version TEXT PRIMARY KEY,\
             applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
         )",
    )
    .execute(pool)
    .await?;

    for (version, migration) in MIGRATIONS {
        let applied = sqlx::query_scalar::<_, String>(
            "SELECT version FROM schema_migrations WHERE version = $1",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;

        if applied.is_some() {
            info!("Migration {} already applied, skipping", version);
            continue;
        }

        info!("Applying migration {}...", version);

        let mut tx = pool.begin().await?;

        for statement in migration.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&mut *tx).await?;
            }
        }

        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(version)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
    }

    info!("Migrations completed successfully");
    Ok(())
}
