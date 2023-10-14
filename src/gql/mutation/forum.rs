use async_graphql::InputObject;
use sqlx::{Postgres, QueryBuilder};

use crate::db::models::forum::{Forum, NewForum, UpdateForum};
use crate::db::models::MaybeEmptyFile;

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
            icon: self._icon.map(MaybeEmptyFile::new),
            banner: self._banner.map(MaybeEmptyFile::new),
        }
    }
}

pub async fn create_forum(
    owner_id: i32,
    forum_name: String,
    display_name: String,
    description: Option<String>,
    pool: &crate::Pool,
) -> anyhow::Result<Forum> {
    let new_forum = NewForum {
        name: &forum_name,
        display_name: &display_name,
        description: description.as_deref(),
        owner_id: owner_id,
    };

    let forum = sqlx::query_as!(
        Forum,
        "
        INSERT INTO forums (name, display_name, description, owner_id)
        VALUES ($1, $2, $3, $4)
        RETURNING *;
        ",
        new_forum.name,
        new_forum.display_name,
        new_forum.description,
        new_forum.owner_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(forum)
}

pub async fn update_forum(
    user_id: i32,
    changes: &UpdateForum,
    pool: &crate::Pool,
) -> anyhow::Result<Forum> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE forums SET ");
    let mut prev = false;
    if let Some(v) = &changes.name {
        builder.push("name = ");
        builder.push_bind(v);
        prev = true;
    }
    if let Some(v) = &changes.display_name {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("display_name = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.description {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("description = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.icon {
        if v.status().await?.updatable() {
            if prev {
                builder.push(", ");
            }
            prev = true;
            builder.push("icon = ");
            builder.push_bind(&v.id);
        } else {
            return Err(anyhow::Error::msg(format!(
                "File with id: {} doesn't exist (icon change)",
                &v.id.clone().unwrap_or("null".into())
            )));
        }
    }
    if let Some(v) = &changes.banner {
        if v.status().await?.updatable() {
            if prev {
                builder.push(", ");
            }
            prev = true;
            builder.push("banner = ");
            builder.push_bind(&v.id);
        } else {
            return Err(anyhow::Error::msg(format!(
                "File with id: {} doesn't exist (banner change)",
                &v.id.clone().unwrap_or("null".into())
            )));
        }
    }
    if let Some(v) = &changes.owner_id {
        if prev {
            builder.push(", ");
        }
        builder.push("owner_id = ");
        builder.push_bind(v);
    }
    builder.push(" WHERE id = ");
    builder.push_bind(changes.id);
    builder.push(" AND owner_id = ");
    builder.push_bind(user_id);
    builder.push(" RETURNING *;");

    let forum = builder.build_query_as::<Forum>().fetch_one(pool).await?;

    Ok(forum)
}
