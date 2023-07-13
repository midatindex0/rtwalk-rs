use std::collections::HashSet;

use once_cell::sync::Lazy;

pub const UNAUTHEMTICATED_MESSAGE: &str = "Unauthenticated request";
pub const RESERVED_USERNAMES: &[&str] = &["admin", "administrator", "moderator", "mod", "system"];
pub const CDN_PATH: &str = "/cdn";
pub static ALLOWED_USERNAME_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    HashSet::from([
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '_',
    ])
});
