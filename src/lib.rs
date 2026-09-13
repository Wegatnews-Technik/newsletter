/// Checks if an E-Mail adress is valid or not:
/// true -> valid false -> invalid
/// # Checked properties
/// - email is not longer than 254 chars
/// - contains exactly one local part and one domain part
/// - both parts are non-empty, domain is at least 3 chars long
/// - only contains the following characters:
///     - 'a'..='z' | '0'..='9' | '-' | '.' | '_' | 'ä' | 'ö' | 'ü' | 'ß'
/// - No trailing underscores, dots and hyphens
/// - No consecutive dots
/// # Panic
/// Panics if email.chars() has more than usize::MAX elements
pub fn check_email(email: &str) -> bool {
    if email.chars().count() >= 254 {
        return false;
    }
    let email: Vec<&str> = email.split("@").collect();
    // Index without Panic
    let (Some(user), Some(domain)) = (email.first(), email.get(1)) else {
        return false;
    };
    // Check lenght of local part and domain part
    if email.len() != 2  // E-Mail consists of two parts
        || user.chars().count() == 0  // local part is non-zero...
        || user.chars().count() > 64  // ...and isn’t to long
        || domain.chars().count() < 3
        || domain.chars().count() > 255
    {
        return false;
    }

    if !allowed_chars(user) || !allowed_chars(domain) {
        return false;
    }

    if !domain.contains(".") {
        return false;
    }

    true
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

#[cfg(test)]
mod tests {
    use crate::check_email;

    #[test]
    fn test_check_email() {
        let valid_emails = [
            "user@example.com",
            "john.doe@example.com",
            "john-doe2@example-domain.com",
            "user@mail.example.com",
            "öäüß@exämple.com",
        ];
        let invalid_emails = [
            "user",
            "user@test@example.com",
            "example@",
            "@example",
            "user-@example.com",
            "user@-example.com",
            "user..name@example.com",
            "user...name@example.com",
            "user'name@example.com",
            "admin'--g@example.com",
        ];

        for mail in valid_emails {
            assert!(check_email(mail), "{mail} should be valid");
        }
        for mail in invalid_emails {
            assert!(!check_email(mail), "{mail} should be invalid");
        }
    }
}
