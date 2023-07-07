use async_graphql::{InputObject, OneofObject};
use diesel::prelude::*;

use super::Page;
use crate::db::models::user::User;
use crate::schema::users::dsl::*;
use crate::search::SearchIndex;

#[derive(InputObject, Default)]
pub struct UserFilter {
    page: Option<Page>,
}

struct RawUserFilter {
    page: Page,
}

impl From<Option<UserFilter>> for RawUserFilter {
    fn from(value: Option<UserFilter>) -> Self {
        let value = value.unwrap_or_default();
        Self {
            page: value.page.unwrap_or_default(),
        }
    }
}

#[derive(OneofObject)]
pub enum UserCriteria {
    Search(String),
    ByUsernames(Vec<String>),
}

pub fn get_users(
    criteria: UserCriteria,
    filter: Option<UserFilter>,
    index: &SearchIndex,
    conn: &mut crate::Conn,
) -> anyhow::Result<Vec<User>> {
    let filter: RawUserFilter = filter.into();
    let x: Vec<User> = match criteria {
        UserCriteria::Search(query) => {
            let results =
                index
                    .user
                    .search(&query, filter.page.offset(), filter.page.per as usize)?;
            let ids = results.ids();
            users.filter(id.eq_any(ids)).load::<User>(conn)?
        }
        UserCriteria::ByUsernames(usernames) => users
            .filter(username.eq_any(usernames))
            .offset(filter.page.offset() as i64)
            .limit(filter.page.per as i64)
            .load::<User>(conn)?,
    };
    Ok(x)
}

pub fn get_user_by_username(uname: &str, conn: &mut crate::Conn) -> anyhow::Result<User> {
    let s = users
        .filter(username.eq(uname))
        .select(User::as_select())
        .get_result(conn);
    Ok(s?)
}
