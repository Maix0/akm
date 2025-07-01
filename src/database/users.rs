use super::Database;

use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;

super::defineID!(UserId => "users");

#[derive(Debug, Clone)]
pub struct TableUsers {
    pub id: UserId,
    pub name: String,
    pub token: SecretString,
}

impl Database {
    pub async fn create_user(&self, name: impl AsRef<str>) -> Result<(UserId, SecretString)> {
        let name = name.as_ref();
        let sha = sha2::Sha256::digest(name);

        let mut token = String::with_capacity(sha.len() * 2);
        for &x in sha.as_slice() {
            use std::fmt::Write;
            write!(&mut token, "{x:02x}").unwrap();
        }

        let query = sqlx::query!(
            "INSERT INTO users ('name', 'token') VALUES (?, ?) RETURNING id, token",
            name,
            token,
        )
        .fetch_one(&self.inner)
        .await?;

        Ok((UserId(query.id), query.token.into()))
    }

    pub async fn fetch_user(&self, id: UserId) -> Result<Option<TableUsers>> {
        let query = sqlx::query!("SELECT * FROM users where id = ? LIMIT 1", id.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|s| TableUsers {
            id: UserId(id.0),
            name: s.name,
            token: s.token.into(),
        }))
    }

    pub async fn remove_user(&self, user: UserId) -> Result<bool> {
        sqlx::query!("DELETE FROM users WHERE id = ?", user.0)
            .execute(&self.inner)
            .await
            .inspect(|s| assert!(s.rows_affected() <= 1, "mutliple user with the same id"))
            .map(|s| s.rows_affected() == 1)
            .map_err(color_eyre::Report::from)
    }

    pub async fn get_user_from_token(&self, token: SecretString) -> Result<Option<UserId>> {
        let t = token.expose_secret();
        sqlx::query!("SELECT id FROM users WHERE token = ? LIMIT 1", t)
            .fetch_optional(&self.inner)
            .await
            .map(|i| i.map(|i| UserId(i.id)))
            .map_err(color_eyre::Report::from)
    }
}
