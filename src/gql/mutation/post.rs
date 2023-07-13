use ::slug::slugify;
use async_graphql::InputObject;
use diesel::ExpressionMethods;
use diesel::{insert_into, RunQueryDsl};

use crate::db::models::post::NewPost;
use crate::db::models::post::Post;
use crate::db::models::post::UpdatePost;
use crate::db::models::File;
use crate::error::PostCreationError;
use crate::schema::posts::dsl::*;

#[derive(InputObject)]
pub struct BasicPostUpdate {
    _id: i32,
    _tags: Option<Vec<String>>,
    _title: Option<String>,
    _content: Option<String>,
    _media: Option<Vec<String>>,
}

impl Into<UpdatePost> for BasicPostUpdate {
    fn into(self) -> UpdatePost {
        let mut _slug;
        if let Some(ref _title) = self._title {
            _slug = Some(slugify(_title));
        } else {
            _slug = None;
        }
        UpdatePost {
            id: self._id,
            tags: self._tags.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(x.into_iter().map(|x| Some(x)).collect())
                }
            }),
            title: self._title,
            slug: _slug,
            stars: None,
            content: self
                ._content
                .map(|x| if x.is_empty() { None } else { Some(x) }),
            media: self._media.map(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(x.into_iter().map(|x| Some(File::new(x))).collect())
                }
            }),
            edited: true,
            edited_at: Some(chrono::Utc::now().naive_utc()),
            poster_id: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_post<'a>(
    _tags: Option<Vec<String>>,
    _title: String,
    _slug: String,
    _content: Option<String>,
    _media: Option<Vec<String>>,
    _forum: i32,
    _poster: i32,
    conn: &mut crate::Conn,
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
                }
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

pub fn update_post(
    user_id: i32,
    changes: &UpdatePost,
    conn: &mut crate::Conn,
) -> anyhow::Result<Post> {
    let x = diesel::update(posts)
        .set(changes)
        .filter(poster_id.eq(user_id))
        .get_result::<Post>(conn)?;
    Ok(x)
}
