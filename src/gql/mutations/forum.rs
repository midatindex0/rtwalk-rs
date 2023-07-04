use diesel::{insert_into, RunQueryDsl};
use log;

use crate::db::models::forum::{Forum, NewForum};
use crate::error::ForumCreationError;
use crate::schema::forums::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

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
