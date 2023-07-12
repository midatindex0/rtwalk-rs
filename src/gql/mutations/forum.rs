use async_graphql::InputObject;
use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use log;

use crate::db::models::forum::{Forum, NewForum, UpdateForum};
use crate::db::models::File;
use crate::error::ForumCreationError;
use crate::schema::forums::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(InputObject)]
pub struct BasicForumUpdate {
    pub _id: i32,
    pub _display_name: Option<String>,
    pub _icon: Option<String>,
    pub _banner: Option<String>,
    pub _description: Option<String>,
}

impl Into<UpdateForum> for BasicForumUpdate {
    fn into(self) -> UpdateForum {
        UpdateForum {
            id: self._id,
            name: None,
            owner_id: None,
            display_name: self._display_name,
            description: self
                ._description
                .map(|x| if x.is_empty() { None } else { Some(x) }),
            icon: self._icon.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(File::new(x))
                }
            }),
            banner: self._banner.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(File::new(x))
                }
            }),
        }
    }
}

pub fn update_forum(user_id: i32, changes: &UpdateForum, conn: &mut Conn) -> anyhow::Result<Forum> {
    let x = diesel::update(forums)
        .set(changes)
        .filter(owner_id.eq(user_id))
        .get_result::<Forum>(conn)?;
    Ok(x)
}

pub fn create_forum<'a>(
    _owner_id: i32,
    _forum_name: String,
    _display_name: String,
    _description: Option<String>,
    conn: &mut Conn,
) -> Result<Forum, ForumCreationError<'a>> {
    let new_forum = NewForum {
        name: &_forum_name,
        display_name: &_display_name,
        description: _description.as_deref(),
        owner_id: _owner_id,
    };
    match insert_into(forums)
        .values(&new_forum)
        .get_result::<Forum>(conn)
    {
        Ok(_forum) => Ok(_forum),
        Err(err) => match err {
            diesel::result::Error::DatabaseError(kind, info) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => Err(
                    ForumCreationError::ForumAlreadyExists("Forum already exists."),
                ),
                _ => {
                    log::error!("{:?}", info);
                    Err(ForumCreationError::InternalError(
                        "Some error occured, try again later.",
                    ))
                }
            },
            _ => {
                log::error!("{:?}", err);
                Err(ForumCreationError::InternalError(
                    "Some error occured, try again later.",
                ))
            }
        },
    }
}
