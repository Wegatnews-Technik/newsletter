use anyhow::Context;
use newsletter::SubscribedEmail;
use rss::Channel;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use tokio::{self, io};
use tracing::Level;

use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

const HOSTNAME: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let rss_content = reqwest::get("https://wegatnews.de/feed.xml")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&rss_content[..])?;
    let email_content = build_email_body(channel);

    let email_secret_file_content =
        fs::read_to_string("/run/secrets/email-config").context("Can't read email config")?;
    let (sender_email, email_password) = parse_email_secret(&email_secret_file_content)?;
    let creds = Credentials::new(sender_email.clone(), email_password);
    let mailer = SmtpTransport::starttls_relay("smtp.office365.com")?
        .credentials(creds)
        .port(587)
        .build();

    let db_password =
        fs::read_to_string("/run/secrets/pg-mailbot").context("Error reading DB secret")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&("postgres://mailbot:".to_owned() + &db_password + "@db/newsletter"))
        .await
        .context("Can’t connect to database")?;
    let rows: Vec<SubscribedEmail> = sqlx::query_as("SELECT * FROM emails LIMIT 100;")
        .fetch_all(&pool)
        .await
        .context("Can’t read from DB")?;

    for mut row in rows {
        let Some(dest_email) = (match row.validate().await {
            Ok(()) => row.address,
            Err(e) => {
                tracing::event!(Level::WARN, ?row.uuid, "Could not send email: {e}");
                continue;
            }
        }) else {
            tracing::event!(Level::ERROR, ?row.uuid, "destination_email is None");
            continue;
        };
        let unsubscribe_link = format!(
            r#"Zum deabonnieren diesen Link klicken: <a href="{HOSTNAME}/unsubscribe?uuid={}">{HOSTNAME}/unsubscribe?uuid={}<a>"#,
            row.uuid.unwrap(),
            row.uuid.unwrap()
        );
        let email_to_send = Message::builder()
            .from(Mailbox::new(
                Some("Wegatnews".to_owned()),
                sender_email.parse()?,
            ))
            .to(Mailbox::new(None, dest_email.parse()?))
            .subject("Neuer Artikel")
            .header(ContentType::TEXT_HTML)
            .body(email_content.clone() + &unsubscribe_link)?;
        mailer.send(&email_to_send)?;
    }

    Ok(())
}

fn build_email_body(channel: Channel) -> String {
    let mut output = String::with_capacity(350);
    let newest_post = channel.items()[0].to_owned();
    if let Some(title) = newest_post.title {
        output += &("<h1>".to_owned() + &title + "</h2>");
    }
    if let Some(description) = newest_post.description {
        output += &description;
    }
    if let Some(link) = newest_post.link {
        output += &("<a href='".to_owned() + &link + "'>ganzer Artikel</a>");
    }
    output
}

fn parse_email_secret(input: &str) -> Result<(String, String), io::Error> {
    let email_config: Vec<&str> = input.split("\n").collect();
    let (Some(account), Some(password)) = (email_config.first(), email_config.get(1)) else {
        return Err(io::Error::other(
            "Email account config is malformed",
        ));
    };
    Ok((account.trim().to_string(), password.trim().to_string()))
}
