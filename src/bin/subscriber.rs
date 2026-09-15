use newsletter;
use sqlx::{pool, postgres::PgPoolOptions};
use std::{fs, io};
use tokio;

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
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@localhost/test"))
        .await?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if newsletter::check_email(&input.trim().to_lowercase()) {
        println!("{input}");
    } else {
        println!("Rejected");
    }
    Ok(())
}
