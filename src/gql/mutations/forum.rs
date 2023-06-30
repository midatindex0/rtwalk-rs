use diesel::{insert_into, RunQueryDsl};
use log;

use crate::db::models::forum::NewForum;
use crate::error::ForumCreationError;
use crate::schema::forums::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn create_forum(
    _owner_id: i32,
    _forum_name: String,
    _description: String,
    conn: &mut Conn,
) -> Result<usize, ForumCreationError> {
    let new_forum = NewForum {
        name: &_forum_name,
        description: Some(&_description),
        owner_id: _owner_id,
    };
    match insert_into(forums).values(&new_forum).execute(conn) {
        Ok(_id) => Ok(_id),
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
