use rusty_paseto::prelude::*;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct AuthUser {
    username: Option<String>,
}

impl AuthUser {
    pub fn to_token<'a>(&self, key: &PasetoSymmetricKey<V4, Local>) -> anyhow::Result<String> {
        Ok(PasetoBuilder::<V4, Local>::default()
            .set_claim(CustomClaim::try_from(("user", self))?)
            .build(key)?)
    }
}
