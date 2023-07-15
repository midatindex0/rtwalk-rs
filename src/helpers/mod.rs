use std::collections::HashSet;

use crate::{constants::{ALLOWED_USERNAME_CHARS, RESERVED_USERNAMES}};

#[macro_export]
macro_rules! spawn_blocking {
    ($val:expr) => {
        actix_rt::task::spawn_blocking(move || $val).await
    };
}

pub fn calculate_password_strength(password: &str, username: &str) -> anyhow::Result<u8> {
    let x = zxcvbn::zxcvbn(password, &[username])?;
    Ok(x.score())
}

pub fn check_reserved_username(username: &str) -> bool {
    RESERVED_USERNAMES.contains(&username)
}

pub fn check_valid_uservane(username: &str) -> bool {
    let username: HashSet<char> = username.chars().collect();

    if ALLOWED_USERNAME_CHARS
        .union(&username)
        .collect::<Vec<_>>()
        .len()
        != ALLOWED_USERNAME_CHARS.len()
    {
        return false;
    }
    true
}