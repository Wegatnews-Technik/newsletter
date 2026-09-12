use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input Error"); // TODO: RISK OF PANIC
    if check_email(&input.trim().to_lowercase()) {
        println!("{input}");
    } else {
        println!("Rejected");
    }
}

fn check_email(email: &str) -> bool {
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
