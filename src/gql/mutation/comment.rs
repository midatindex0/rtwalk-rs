use async_graphql::InputObject;
use sqlx::{query, Postgres, QueryBuilder};

use crate::db::models::comment::{Comment, UpdateComment};
use crate::db::models::{comment::NewComment, FileList};

#[derive(InputObject)]
pub struct BasicCommentUpdate {
    id: i32,
    content: Option<String>,
    media: Option<Vec<String>>,
}

impl Into<UpdateComment> for BasicCommentUpdate {
    fn into(self) -> UpdateComment {
        UpdateComment {
            id: self.id,
            user_id: None,
            content: self.content.map(|x| {
                if x.is_empty() {
                    "User cleared this comment".to_string()
                } else {
                    x
                }
            }),
            media: self.media.map(FileList::new),
            edited: true,
            edited_at: Some(chrono::Utc::now().naive_utc()),
        }
    }
}

pub async fn create_comment(
    _user_id: i32,
    _post_id: i32,
    _forum_id: i32,
    _parent_id: Option<i32>,
    _content: String,
    _media: Option<Vec<String>>,
    pool: &crate::Pool,
) -> anyhow::Result<Comment> {
    let new_comment = NewComment {
        user_id: _user_id,
        post_id: _post_id,
        forum_id: _forum_id,
        parent_id: _parent_id,
        content: _content,
        media: _media,
    };

    let comment = sqlx::query_as!(
        Comment,
        "
        INSERT INTO comments (user_id, post_id, forum_id, parent_id, media, content)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *;
        ",
        new_comment.user_id,
        new_comment.post_id,
        new_comment.forum_id,
        new_comment.parent_id,
        new_comment.media.as_ref().map(Vec::as_slice),
        new_comment.content,
    )
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

pub async fn update_comment(
    user_id: i32,
    changes: &UpdateComment,
    pool: &crate::Pool,
) -> anyhow::Result<Comment> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE posts SET ");
    let mut prev = false;

    if let Some(v) = &changes.user_id {
        builder.push("user_id = ");
        builder.push_bind(v);
        prev = true;
    }
    if let Some(v) = &changes.content {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("content = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.media {
        let files = v.ids();
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("media = ");
        builder.push_bind(files);
    }
    if prev {
        builder.push(", ");
    }
    builder.push("edited = ");
    builder.push_bind(changes.edited);
    builder.push(", edited_at = ");
    builder.push_bind(changes.edited_at);

    builder.push(" WHERE id = ");
    builder.push_bind(changes.id);
    builder.push(" AND user_id = ");
    builder.push_bind(user_id);
    builder.push(" RETURNING *;");

    let comment = builder.build_query_as::<Comment>().fetch_one(pool).await?;

    Ok(comment)
}
