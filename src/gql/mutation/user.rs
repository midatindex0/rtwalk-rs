use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_graphql::InputObject;
use sqlx::{QueryBuilder, Postgres};

use crate::db::models::user::User;
use crate::db::models::user::{NewUser, UpdateUser};
use crate::db::models::MaybeEmptyFile;

#[derive(InputObject)]
pub struct BasicUserUpdate {
    pub _display_name: Option<String>,
    pub _bio: Option<String>,
    pub _pfp: Option<String>,
    pub _banner: Option<String>,
}

impl Into<UpdateUser> for BasicUserUpdate {
    fn into(self) -> UpdateUser {
        UpdateUser {
            id: 0,
            username: None,
            password: None,
            display_name: self._display_name,
            bio: self._bio.map(|x| if x.is_empty() { None } else { Some(x) }),
            pfp: self._pfp.map(MaybeEmptyFile::new),
            banner: self._banner.map(MaybeEmptyFile::new),
            admin: None,
        }
    }
}

pub async fn create_user(
    _username: String,
    _password: String,
    pool: &crate::Pool,
) -> anyhow::Result<User> {
    let new_user = NewUser {
        username: &_username,
        password: &_password,
        display_name: &_username,
    };

    let user = sqlx::query_as!(
        User,
        "
        INSERT INTO users (username, password, display_name)
        VALUES ($1, $2, $3)
        RETURNING *;
        ",
        new_user.username,
        new_user.password,
        new_user.display_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn update_user(changes: &UpdateUser, pool: &crate::Pool) -> anyhow::Result<User> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE users SET ");
    let mut prev = false;
    if let Some(v) = &changes.username {
        builder.push("username = ");
        builder.push_bind(v);
        prev = true;
    }
    if let Some(v) = &changes.password {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("password = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.display_name {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("display_name = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.bio {
        if prev {
            builder.push(", ");
        }
        prev = true;
        builder.push("bio = ");
        builder.push_bind(v);
    }
    if let Some(v) = &changes.pfp {
        if v.status().await?.updatable() {
            if prev {
                builder.push(", ");
            }
            prev = true;
            builder.push("pfp = ");
            builder.push_bind(&v.id);
        } else {
            return Err(anyhow::Error::msg(format!(
                "File with id: {} doesn't exist (pfp change)",
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
    if let Some(v) = &changes.admin {
        if prev {
            builder.push(", ");
        }
        builder.push("admin = ");
        builder.push_bind(v);
    }
    builder.push(" WHERE id = ");
    builder.push_bind(changes.id);
    builder.push(" RETURNING *;");

    let user = builder.build_query_as::<User>().fetch_one(pool).await?;

    Ok(user)
}

pub async fn verify_user<'a>(
    _username: &str,
    _password: &str,
    pool: &crate::Pool,
    hasher: &Argon2<'_>,
) -> anyhow::Result<(bool, User)> {
    let _user: User = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", _username)
        .fetch_one(pool)
        .await?;

    let parsed_hash = PasswordHash::new(&_user.password)
        .map_err(|_| anyhow::Error::msg("Some interbal error occured"))?;

    Ok((
        hasher
            .verify_password(_password.as_bytes(), &parsed_hash)
            .is_ok(),
        _user,
    ))
}
