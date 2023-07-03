use diesel::{insert_into, RunQueryDsl};

use crate::db::models::post::NewPost;
use crate::db::models::post::Post;
use crate::error::PostCreationError;
use crate::schema::posts::dsl::*;

type Conn = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[allow(clippy::too_many_arguments)]
pub fn create_post<'a>(
    _tags: Option<Vec<String>>,
    _title: String,
    _slug: String,
    _content: Option<String>,
    _media: Option<Vec<String>>,
    _forum: i32,
    _poster: i32,
    conn: &mut Conn,
) -> Result<Post, PostCreationError<'a>> {
    let new_post = NewPost {
        tags: _tags,
        title: _title,
        slug: _slug,
        content: _content,
        media: _media,
        forum_id: _forum,
        poster_id: _poster,
    };
    match insert_into(posts)
        .values(&new_post)
        .get_result::<Post>(conn)
    {
        Ok(_post) => Ok(_post),
        Err(err) => match err {
            diesel::result::Error::DatabaseError(kind, info) => match kind {
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    Err(PostCreationError::ForumNotFound("Forum does not exist"))
                },
                _ => {
                    log::error!("{:?}", info);
                    Err(PostCreationError::InternalError(
                        "Some error occured, try again later.",
                    ))
                }
            },
            _ => {
                log::error!("{:?}", err);
                Err(PostCreationError::InternalError(
                    "Some error occured, try again later.",
                ))
            }
        },
    }
}
