use std::collections::HashSet;

use once_cell::sync::Lazy;

use crate::constants::RESERVED_USERNAMES;

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

static ALLOWED_USERNAME_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    HashSet::from([
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '_',
    ])
});

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
