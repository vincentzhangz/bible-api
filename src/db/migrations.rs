use sqlx::PgPool;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    let migration_sql = include_str!("../../migrations/001_initial.sql");

    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    info!("Migrations completed successfully");
    Ok(())
}
