use anyhow::Result;
use sqlx::PgPool;

pub mod models;
pub mod schema;

pub async fn connect(url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
