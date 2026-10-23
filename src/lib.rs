use check_if_email_exists::{self, CheckEmailInputBuilder, Reachable};
use serde::Deserialize;
use std::{env, time::Duration};
use thiserror::Error;
use tracing::Level;
use uuid::Uuid;

pub struct Config {
    pub base_url: String,
    pub ttl: Duration,
    pub check_email_exists: bool,
}
impl Config {
    pub fn read() -> Config {
        let base_url = match env::var("BASE_URL") {
            Ok(var) => var,
            Err(_) => {
                tracing::event!(Level::ERROR, "Could not read environment variable");
                String::new()
            }
        };
        let (hours, minutes) = (
            env::var("TTL_HOURS").unwrap_or(String::from("0")),
            env::var("TTL_MINUTES").unwrap_or(String::from("0")),
        );
        let hours = match hours.parse() {
            Ok(hours) => hours,
            _ => {
                tracing::event!(
                    Level::ERROR,
                    "invalid configuration for email verification TTL hours"
                );
                0
            }
        };
        let minutes = match minutes.parse() {
            Ok(minutes) => minutes,
            _ => {
                tracing::event!(
                    Level::ERROR,
                    "invalid configuration for email verification TTL minutes"
                );
                0
            }
        };
        let mut ttl = Duration::from_hours(hours) + Duration::from_mins(minutes);
        if ttl == Duration::ZERO {
            ttl = Duration::from_mins(30);
        }

        Config {
            base_url,
            ttl,
            check_email_exists: env::var("CHECK_EMAIL_EXISTS").is_ok(),
        }
    }
}

#[derive(Error, Debug)]
pub enum EmailValidationError {
    // parsing errors
    #[error("Email to long")]
    ToLong,
    #[error("Email is missing some parts")]
    ToShort,
    #[error("Email contains multiple '@' symbols")]
    MultipleAt,
    #[error("Domain is missing a dot(.)")]
    DomainMissingDot,
    #[error("Invalid Character(s)")]
    InvalidCharacter,
    // email_exists errors
    #[error("E-Mail is risky")]
    EmailRisky,
    #[error("E-Mail does not exist")]
    EmailInvalid,
    #[error("Could not check E-Mail: {0}")]
    Unknown(String),
}

#[derive(Deserialize, sqlx::FromRow)]
pub struct SubscribedEmail {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub uuid: Option<Uuid>,
}

impl SubscribedEmail {
    /// Checks if an E-Mail address is valid or not:
    /// true -> valid false -> invalid
    /// **This takes ownership over email**
    /// # Checked properties
    /// - email is not longer than 254 chars
    /// - contains exactly one local part and one domain part
    /// - both parts are non-empty, domain is at least 3 chars long
    /// - only contains the following characters:
    ///     - 'a'..='z' | '0'..='9' | '-' | '.' | '_' | 'ä' | 'ö' | 'ü' | 'ß'
    /// - No trailing underscores, dots and hyphens
    /// - No consecutive dots
    /// - Check that E-Mail exists and is not a disposable address(via crate `check-if-email-exists`)
    /// # Panic
    /// Panics if email.chars() has more than usize::MAX elements
    pub async fn validate(&mut self, config: &Config) -> Result<(), EmailValidationError> {
        self.check_email_syntax()?;
        if config.check_email_exists {
            let Some(email) = self.address.clone() else {
                return Err(EmailValidationError::ToShort);
            };
            Self::email_exists(&email).await?;
        }
        Ok(())
    }

    fn check_email_syntax(&mut self) -> Result<(), EmailValidationError> {
        let Some(email) = &self.address else {
            return Err(EmailValidationError::ToShort);
        };
        if email.chars().count() >= 254 {
            self.address = None;
            return Err(EmailValidationError::ToLong);
        }
        let email_parts: Vec<&str> = email.split("@").collect();

        // Index without Panic
        let (Some(user), Some(domain)) = (email_parts.first(), email_parts.get(1)) else {
            self.address = None;
            return Err(EmailValidationError::ToShort);
        };

        if email_parts.len() != 2 {
            self.address = None;
            return Err(EmailValidationError::MultipleAt);
        }
        if user.chars().count() > 64 || domain.chars().count() > 255 {
            self.address = None;
            return Err(EmailValidationError::ToLong);
        }
        if user.chars().count() == 0 || domain.chars().count() < 3 {
            self.address = None;
            return Err(EmailValidationError::ToShort);
        }

        if !Self::allowed_chars(user.trim()) || !Self::allowed_chars(domain.trim()) {
            self.address = None;
            return Err(EmailValidationError::InvalidCharacter);
        }

        if !domain.contains(".") {
            self.address = None;
            return Err(EmailValidationError::DomainMissingDot);
        }

        self.address = Some(user.trim().to_string() + "@" + domain.trim());
        Ok(())
    }

    fn allowed_chars(string: &str) -> bool {
        if !string
            .trim()
            .chars()
            .all(|x| matches!(x, 'a'..='z' | '0'..='9' | '-' | '.' | '_' | 'ä' | 'ö' | 'ü' | 'ß'))
        {
            return false;
        }

        // Consecutive dots
        let mut dot = false;
        for char in string.chars() {
            if char == '.' {
                if dot {
                    return false;
                }
                dot = true;
            } else {
                dot = false;
            }
        }

        // Check for trailing special characters
        if let (Some(first), Some(last)) = (string.chars().next(), string.chars().next_back()) {
            !matches!(first, '-' | '.' | '_') && !matches!(last, '-' | '.' | '_')
        } else {
            false
        }
    }

    async fn email_exists(email: &str) -> Result<(), EmailValidationError> {
        let input = match CheckEmailInputBuilder::default()
            .to_email(email.to_string())
            .build()
        {
            Ok(input) => input,
            Err(e) => return Err(EmailValidationError::Unknown(format!("{:?}", e))),
        };

        let result = check_if_email_exists::check_email(&input).await;

        match result.is_reachable {
            Reachable::Safe => Ok(()),
            Reachable::Risky => Err(EmailValidationError::EmailRisky),
            Reachable::Invalid => Err(EmailValidationError::EmailInvalid),
            Reachable::Unknown => {
                if let Err(e) = result.mx {
                    Err(EmailValidationError::Unknown(e.to_string()))
                } else if let Err(e) = result.smtp {
                    Err(EmailValidationError::Unknown(e.to_string()))
                } else if let Err(e) = result.misc {
                    Err(EmailValidationError::Unknown(e.to_string()))
                } else {
                    Err(EmailValidationError::Unknown(String::from(
                        "Unknown problem",
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::SubscribedEmail;

    #[test]
    fn test_check_email_syntax() {
        let valid_emails = [
            "user@example.com".to_string(),
            "john.doe@example.com".to_string(),
            "john-doe2@example-domain.com".to_string(),
            "user@mail.example.com".to_string(),
            "öäüß@exämple.com".to_string(),
        ];
        let invalid_emails = [
            "user".to_string(),
            "user@test@example.com".to_string(),
            "example@".to_string(),
            "@example".to_string(),
            "user-@example.com".to_string(),
            "user@-example.com".to_string(),
            "user..name@example.com".to_string(),
            "user...name@example.com".to_string(),
            "user'name@example.com".to_string(),
            "admin'--g@example.com".to_string(),
        ];

        for mail in valid_emails {
            let mut subscribed_email = SubscribedEmail {
                address: Some(mail.clone()),
                uuid: None,
            };
            assert_matches!(
                subscribed_email.check_email_syntax(),
                Ok(_),
                "{mail} should be valid"
            );
        }
        for mail in invalid_emails {
            let mut subscribed_email = SubscribedEmail {
                address: Some(mail.clone()),
                uuid: None,
            };
            assert_matches!(
                subscribed_email.check_email_syntax(),
                Err(_),
                "{mail} should be invalid"
            )
        }
    }
}
