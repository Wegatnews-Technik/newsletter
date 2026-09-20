use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::{fs, io};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_password =
        fs::read_to_string("/run/secrets/pg-subscriber").context("Error reading DB secret")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    input = newsletter::parse(input).await?;
    sqlx::query("INSERT INTO emails VALUES ($1)")
        .bind(&input)
        .execute(&pool)
        .await?;
    Ok(())
}
