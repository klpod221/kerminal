pub mod sqlite;

// Migration trait
use crate::database::error::DatabaseResult;
use async_trait::async_trait;

#[async_trait]
pub trait Migration {
    fn version(&self) -> u32;
    fn description(&self) -> &str;
    async fn up(&self, db: &dyn crate::database::Database) -> DatabaseResult<()>;
    async fn down(&self, db: &dyn crate::database::Database) -> DatabaseResult<()>;
}

pub struct MigrationRunner {
    migrations: Vec<Box<dyn Migration + Send + Sync>>,
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn add_migration(&mut self, migration: Box<dyn Migration + Send + Sync>) {
        self.migrations.push(migration);
    }

    pub async fn run_migrations(&self, db: &dyn crate::database::Database) -> DatabaseResult<()> {
        // Sort migrations by version
        let mut migrations = self.migrations.iter().collect::<Vec<_>>();
        migrations.sort_by_key(|m| m.version());

        for migration in migrations {
            migration.up(db).await?;
        }

        Ok(())
    }
}
