use anyhow::Context;
use axum::{
    Router,
    extract::{Form, State},
    response::Html,
    routing::post,
};
use newsletter::SubscribedEmail;
use reqwest::StatusCode;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{fs, sync::Arc};
use tracing::Level;
use uuid::Uuid;

struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_password =
        fs::read_to_string("/run/secrets/pg-subscriber").context("Error reading DB secret")?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;
    tracing::event!(Level::INFO, "established DB connection");

    let state = Arc::new(AppState { db });
    let app = Router::new()
        .route("/subscribe", post(handle_subscribe))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_subscribe(
    state: State<Arc<AppState>>,
    Form(params): Form<SubscribedEmail>,
) -> (StatusCode, Html<String>) {
    let Some(email) = params.email else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Html("Es wurde keine E-Mailadresse angegeben".to_string()),
        );
    };
    if email.trim() == "" {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Html("Es wurde keine E-Mailadresse angegeben".to_string()),
        );
    }
    let email = match newsletter::parse(email.clone()).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Html(format!("Es ist ein Fehler aufgetreten: {e}")),
            );
        }
    };
    let uuid = sqlx::types::Uuid::from_u128(Uuid::now_v7().to_u128_le());
    match sqlx::query("INSERT INTO emails VALUES ($1, $2)")
        .bind(uuid)
        .bind(&email)
        .execute(&state.db)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Es ist ein Fehler aufgetreten: {e}")),
            );
        }
    }
    tracing::event!(Level::INFO, "add email: {email} to newsletter");

    (StatusCode::OK, Html("Success".to_string()))
}
