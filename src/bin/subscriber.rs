use anyhow::{Context, Ok};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{fs, io};
use tracing::Level;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_password =
        fs::read_to_string("/run/secrets/pg-subscriber").context("Error reading DB secret")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;
    tracing::event!(Level::INFO, "established DB connection");

    loop {
        match write_input_to_db(&pool).await {
            anyhow::Result::Ok(false) => {
                tracing::event!(Level::INFO, "finished handling inputs");
                return Ok(());
            }
            anyhow::Result::Ok(true) => (),
            Err(e) => tracing::event!(Level::WARN, "error accepting subscription [{e}]"),
        }
    }
}

async fn write_input_to_db(sql_connection: &Pool<Postgres>) -> anyhow::Result<bool> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if input.trim() == "" {
        return Ok(false);
    }
    input = newsletter::parse(input).await?;
    let uuid = sqlx::types::Uuid::from_u128(Uuid::now_v7().to_u128_le());
    sqlx::query("INSERT INTO emails VALUES ($1, $2)")
        .bind(uuid)
        .bind(&input)
        .execute(sql_connection)
        .await?;
    tracing::event!(Level::INFO, "add email: {input} to newsletter");
    Ok(true)
}
