mod entities;

pub use entities::{config, report, user};
pub use sea_orm::DbErr;
use sea_orm::{ConnectOptions, DatabaseConnection};
use tracing::log::LevelFilter;

#[derive(Clone, Debug)]
pub struct Database {
    pub conn: DatabaseConnection,
}

pub async fn initialize() -> Result<Database, DbErr> {
    let dsn = std::env::var("DATABASE_URL")
        .map_err(|_| DbErr::Custom("missing DATABASE_URL".to_owned()))?;
    let mut connect_options = ConnectOptions::new(&dsn);
    connect_options
        .acquire_timeout(std::time::Duration::from_secs(15))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Debug);

    let conn = sea_orm::Database::connect(connect_options).await?;
    Ok(Database { conn })
}
