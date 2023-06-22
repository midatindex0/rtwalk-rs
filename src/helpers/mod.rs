pub fn calculate_password_strength(password: &str) -> i32 {
    let mut uppercase = 0;
    let mut lowercase = 0;
    let mut numeric = 0;
    let mut special = 0;
    for ch in password.chars() {
        if ch.is_ascii_lowercase() {
            lowercase = 1;
        } else if ch.is_ascii_uppercase() {
            uppercase = 1;
        } else if ch.is_numeric() {
            numeric = 1;
        } else {
            special = 1;
        }
    }

    uppercase + lowercase + numeric + special
}

pub fn check_reserved_username(_username: &str) -> bool {
    false
}
