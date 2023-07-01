use rusty_paseto::prelude::*;
use serde::Serialize;
use std::convert::TryFrom;

#[derive(Serialize, Default, Debug)]
pub struct AuthUser {
    pub username: Option<String>,
    pub version: i32,
}

impl AuthUser {
    pub fn to_token(&self, key: &PasetoSymmetricKey<V4, Local>) -> anyhow::Result<String> {
        Ok(PasetoBuilder::<V4, Local>::default()
            .set_claim(CustomClaim::try_from(("user", self))?)
            .set_no_expiration_danger_acknowledged()
            .build(key)?)
    }
}
