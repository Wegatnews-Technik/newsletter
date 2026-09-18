use sqlx::postgres::PgPoolOptions;
use std::{fs, io};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_password = match fs::read_to_string("/run/secrets/pg-subscriber") {
        Ok(value) => value,
        Err(e) => {
            eprint!("Error reading DB password: {}", e);
            return Ok(()); // TODO: proper error handling
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@db/newsletter"))
        .await?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if newsletter::check_email_syntax(&input.trim().to_lowercase()) {
        match newsletter::email_exists(input.trim()).await {
            Ok(()) => {
                sqlx::query("INSERT INTO emails VALUES ($1)")
                    .bind(&input)
                    .execute(&pool)
                    .await?; // TODO: proper error handling
            }
            Err(e) => eprintln!("{e} Input: {}", input),
        };
    } else {
        println!("Rejected");
    }
    Ok(())
}
