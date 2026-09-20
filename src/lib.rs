use check_if_email_exists::{self, CheckEmailInputBuilder, Reachable};
use thiserror::Error;

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
pub async fn parse(mut email: String) -> Result<String, EmailValidationError> {
    check_email_syntax(&mut email)?;
    email_exists(&email).await?;
    Ok(email)
}

fn check_email_syntax(email: &mut String) -> Result<(), EmailValidationError> {
    if email.chars().count() >= 254 {
        return Err(EmailValidationError::ToLong);
    }
    let email_parts: Vec<&str> = email.split("@").collect();

    // Index without Panic
    let (Some(user), Some(domain)) = (email_parts.first(), email_parts.get(1)) else {
        return Err(EmailValidationError::ToShort);
    };

    if email_parts.len() != 2 {
        return Err(EmailValidationError::MultipleAt);
    }
    if user.chars().count() > 64 || domain.chars().count() > 255 {
        return Err(EmailValidationError::ToLong);
    }
    if user.chars().count() == 0 || domain.chars().count() < 3 {
        return Err(EmailValidationError::ToShort);
    }

    if !allowed_chars(user.trim()) || !allowed_chars(domain.trim()) {
        return Err(EmailValidationError::InvalidCharacter);
    }

    if !domain.contains(".") {
        return Err(EmailValidationError::DomainMissingDot);
    }

    *email = user.trim().to_string() + "@" + domain.trim();
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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crate::check_email_syntax;

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

        for mut mail in valid_emails {
            assert_matches!(
                check_email_syntax(&mut mail),
                Ok(_),
                "{mail} should be valid"
            );
        }
        for mut mail in invalid_emails {
            assert_matches!(
                check_email_syntax(&mut mail),
                Err(_),
                "{mail} should be invalid"
            )
        }
    }
}
