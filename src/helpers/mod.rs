use std::collections::HashSet;

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

pub fn check_valid_uservane(username: &str) -> bool {
    let allowed_chars = HashSet::from([
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '_',
    ]);
    let username: HashSet<char> = username.chars().collect();

    if allowed_chars.union(&username).collect::<Vec<_>>().len() > allowed_chars.len() {
        return false;
    }
    true
}
