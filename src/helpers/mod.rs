use crate::constants::RESERVED_USERNAMES;

#[macro_export]
macro_rules! spawn_blocking {
    ($val:expr) => {
        actix_rt::task::spawn_blocking(move || $val).await
    };
}

pub fn calculate_password_strength(_password: &str) -> i32 {
    let mut uppercase = 0;
    let mut lowercase = 0;
    let mut numeric = 0;
    let mut special = 0;
    for ch in _password.chars() {
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

pub fn check_reserved_username(username: &str) -> bool {
    RESERVED_USERNAMES.contains(&username)
}

pub fn check_valid_uservane(username: &str) -> bool {
    for ch in username.chars() {
        if !(ch.is_alphanumeric() || ch == '_') || ch.is_ascii_lowercase() {
            return false;
        }
    }
    true
}
