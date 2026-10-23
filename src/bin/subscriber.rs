use anyhow::Context;
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{Form, Query, State},
    routing::{get, post},
};
use moka::future::{Cache, CacheBuilder};
use newsletter::{Config, EmailValidationError, SubscribedEmail};
use reqwest::StatusCode;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{fs, sync::Arc};
use tracing::Level;
use uuid::Uuid;

struct AppState {
    db: PgPool,
    config: Config,
    unverified_emails: Cache<Uuid, String>,
}

trait TemplateTracing {
    fn trace_error<T: std::fmt::Display>(&self, error: T) -> &'static str {
        tracing::event!(Level::ERROR, "{error}");
        ""
    }

    fn trace_error_debug_print<T: std::fmt::Debug>(&self, error: T) -> &'static str {
        tracing::event!(Level::ERROR, "{:?}", error);
        ""
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "subscribe.html")]
enum SubscribeTemplate {
    Success,
    ValidationError(EmailValidationError),
}

#[derive(Template, WebTemplate)]
#[template(path = "verify.html")]
enum VerifyTemplate {
    Success,
    DBError(sqlx::Error),
    InvalidLink,
}
impl TemplateTracing for VerifyTemplate {}

#[derive(Template, WebTemplate)]
#[template(path = "unsubscribe.html")]
enum UnsubscribeTemplate {
    Success,
    DBError(sqlx::Error),
}
impl TemplateTracing for UnsubscribeTemplate {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::read();

    let db_password =
        fs::read_to_string("/run/secrets/pg-subscriber").context("Error reading DB secret")?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://subscriber:".to_owned() + &db_password[..] + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;
    tracing::event!(Level::INFO, "established DB connection");

    let unverified_emails = CacheBuilder::new(config.unverified_limit)
        .time_to_live(config.ttl)
        .build();
    let state = Arc::new(AppState {
        db,
        config,
        unverified_emails,
    });
    let app = Router::new()
        .route("/subscribe", post(handle_subscribe))
        .route("/verify", get(handle_verify))
        .route("/unsubscribe", get(handle_unsubscribe))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_subscribe(
    state: State<Arc<AppState>>,
    Form(mut params): Form<SubscribedEmail>,
) -> (StatusCode, SubscribeTemplate) {
    match params.validate(&state.config).await {
        Ok(()) => (),
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                SubscribeTemplate::ValidationError(e),
            );
        }
    };
    let uuid = sqlx::types::Uuid::from_u128(Uuid::now_v7().to_u128_le());
    tracing::event!(Level::INFO, "{uuid}");

    state
        .unverified_emails
        .insert(uuid, params.address.expect("address was validated before"))
        .await;

    (StatusCode::OK, SubscribeTemplate::Success)
}

async fn handle_verify(
    state: State<Arc<AppState>>,
    Form(mut params): Form<SubscribedEmail>,
) -> (StatusCode, VerifyTemplate) {
    let Some(uuid) = params.uuid else {
        return (StatusCode::BAD_REQUEST, VerifyTemplate::InvalidLink);
    };
    params.address = state.unverified_emails.remove(&uuid).await;

    match params.validate(&state.config).await {
        Ok(()) => (),
        Err(_) => {
            return (StatusCode::BAD_REQUEST, VerifyTemplate::InvalidLink);
        }
    }

    match sqlx::query("INSERT INTO emails VALUES ($1, $2)")
        .bind(params.uuid)
        .bind(&params.address)
        .execute(&state.db)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                VerifyTemplate::DBError(e),
            );
        }
    }
    if let Some(email) = params.address {
        tracing::event!(Level::INFO, "add email: {email} to newsletter");
    } else {
        tracing::event!(
            Level::ERROR,
            "add NULL to Database. UUID: {:?}",
            params.uuid
        )
    }
    (StatusCode::OK, VerifyTemplate::Success)
}

async fn handle_unsubscribe(
    state: State<Arc<AppState>>,
    to_delete: Query<SubscribedEmail>,
) -> (StatusCode, UnsubscribeTemplate) {
    match sqlx::query_as::<_, SubscribedEmail>(
        "DELETE FROM emails WHERE uuid=$1 RETURNING uuid, address",
    )
    .bind(to_delete.uuid)
    .fetch_one(&state.db)
    .await
    {
        Ok(subscriber) => {
            tracing::event!(Level::INFO, "deleted email: {:?}", subscriber.address);
            (StatusCode::OK, UnsubscribeTemplate::Success)
        }
        Err(e) => (StatusCode::BAD_REQUEST, UnsubscribeTemplate::DBError(e)),
    }
}
