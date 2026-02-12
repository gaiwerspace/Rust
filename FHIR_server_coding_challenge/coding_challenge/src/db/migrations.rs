use sqlx::postgres::PgPool;
use std::fs;
use std::path::Path;

/// Run all database migrations in order
pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let migrations_dir = "migrations";
    
    if !Path::new(migrations_dir).exists() {
        return Err(format!("Migrations directory not found: {}", migrations_dir).into());
    }

    // Read all migration files
    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "sql")
                .unwrap_or(false)
                .and_then(|_| e.file_name().to_str().map(|n| !n.starts_with("run_")))
                .unwrap_or(false)
        })
        .collect();

    // Sort by filename to ensure order
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        
        println!("Running migration: {}", filename);
        
        let sql = fs::read_to_string(&path)?;
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| {
                format!("Migration {} failed: {}", filename, e)
            })?;
    }

    println!("All migrations completed successfully!");
    Ok(())
}

/// Check if migrations table exists (for tracking applied migrations)
pub async fn init_migrations_tracker(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Record a migration as applied
pub async fn record_migration(
    pool: &PgPool,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("INSERT INTO migrations (name) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}
