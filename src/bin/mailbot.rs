use anyhow::Context;
use sqlx::{Row, postgres::PgPoolOptions};
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_password =
        fs::read_to_string("/run/secrets/pg-mailbot").context("Error reading DB secret")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://mailbot:".to_owned() + &db_password + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;

    let rows = sqlx::query("SELECT * FROM emails LIMIT 100;")
        .fetch_all(&pool)
        .await
        .context("Can’t read from DB")?;

    for row in rows {
        let email: String = row.try_get("emails").context("Invalid DB scheme")?;
        let email = newsletter::parse(email).await?;
    }

    Ok(())
}
