use ::slug::slugify;
use async_graphql::InputObject;
use sqlx::Postgres;
use sqlx::QueryBuilder;

use crate::db::models::post::NewPost;
use crate::db::models::post::Post;
use crate::db::models::post::UpdatePost;
use crate::db::models::FileList;

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
                    Some(x.into_iter().map(|x| x).collect())
                }
            }),
            title: self._title,
            slug: _slug,
            stars: None,
            content: self
                ._content
                .map(|x| if x.is_empty() { None } else { Some(x) }),
            media: self._media.map(FileList::new),
            edited: true,
            edited_at: Some(chrono::Utc::now().naive_utc()),
            poster_id: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_post<'a>(
    tags: Option<Vec<String>>,
    title: String,
    slug: String,
    content: Option<String>,
    media: Option<Vec<String>>,
    forum: i32,
    poster: i32,
    pool: &crate::Pool,
) -> anyhow::Result<Post> {
    if let Some(t) = &media {
        let files = FileList::new(t.clone()).files;
        if let Some(files) = files {
            for file in files {
                if !file.status().await?.insertable() {
                    return Err(anyhow::Error::msg(format!(
                        "File with id: {} doesn't exist (media upload)",
                        &file.id.clone().unwrap_or("null".into())
                    )));
                }
            }
        }
    }

    let new_post = NewPost {
        tags,
        title,
        slug,
        content,
        media,
        forum_id: forum,
        poster_id: poster,
    };

    let post = sqlx::query_as!(
        Post,
        "
        INSERT INTO posts (title, slug, content, tags, media, forum_id, poster_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *;
        ",
        new_post.title,
        new_post.slug,
        new_post.content,
        new_post.tags.as_ref().map(Vec::as_slice),
        new_post.media.as_ref().map(Vec::as_slice),
        new_post.forum_id,
        new_post.poster_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(post)
}

pub async fn update_post(
    user_id: i32,
    changes: &UpdatePost,
    pool: &crate::Pool,
) -> anyhow::Result<Post> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE forums SET ");
    let mut prev = false;
    if let Some(v) = &changes.tags {
        builder.push("tags = ");
        builder.push_bind(v.as_ref().map(Vec::as_slice));
        prev = true;
    }
    if let Some(v) = &changes.stars {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("stars = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.title {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("title = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.slug {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("slug = ");
        builder.push_bind(v);
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
    if let Some(v) = &changes.poster_id {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("poster_id = ");
        builder.push_bind(v);
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
    builder.push(" AND poster_id = ");
    builder.push_bind(user_id);
    builder.push(" RETURNING *;");

    let post = builder.build_query_as::<Post>().fetch_one(pool).await?;

    Ok(post)
}
