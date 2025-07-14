use super::Database;
use super::DateTime;

use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;

super::defineID!(KeyId => "keys");

#[derive(Debug, Clone)]
pub struct TableKeys {
    pub id: KeyId,
    pub name: String,
    pub description: String,
    pub key: Option<String>,
    pub rotate_at: Option<DateTime>,
    pub rotate_with: Option<String>,
}

impl Database {
    pub async fn create_key(
        &self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        key: Option<String>,
        update_at: Option<DateTime>,
        update_with: Option<String>,
    ) -> Result<KeyId> {
        let name = name.as_ref();
        let desc = desc.as_ref();

        let k = key.as_ref().map(String::as_str);

        let u_at = update_at.map(|d| d.timestamp());
        let u_with = update_with.as_deref();

        let query = sqlx::query!(
            "INSERT INTO keys ('name', 'description', 'apiKey', 'rotateAt', 'rotateWith') VALUES (?, ?, ?, ?, ?) RETURNING id",
            name,
            desc,
            k,
            u_at,
            u_with
        ).fetch_one(&self.inner).await?;

        Ok(KeyId(query.id))
    }

    pub async fn fetch_key(&self, key: KeyId) -> Result<Option<TableKeys>> {
        let query = sqlx::query!("SELECT * FROM keys WHERE id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableKeys {
            id: KeyId(s.id),
            name: s.name,
            description: s.description,
            key: s.apiKey,
            rotate_at: s.rotateAt.map(DateTime::from_timestamp_nanos),
            rotate_with: s.rotateWith,
        }))
    }

    pub async fn remove_key(&self, key: KeyId) -> Result<bool> {
        sqlx::query!("DELETE FROM keys WHERE id = ?", key.0)
            .execute(&self.inner)
            .await
            .inspect(|s| assert!(s.rows_affected() <= 1, "mutliple key with the same id"))
            .map(|s| s.rows_affected() == 1)
            .map_err(color_eyre::Report::from)
    }

    pub async fn get_all_keys_from_client(
        &self,
        client: super::clients::ClientId,
    ) -> Result<Vec<super::keys::TableKeys>> {
        sqlx::query!(
            "SELECT keys.* FROM keys INNER JOIN clients_key ON clients_key.keyID == keys.id WHERE clients_key.clientID = ?",
            client.0
        ).fetch_all(&self.inner)
        .await
        .map_err(color_eyre::Report::from)
        .map(|v| v.into_iter().map(|r| TableKeys {
            id: KeyId(r.id),
            name: r.name,
            description: r.description,
            key: r.apiKey,
            rotate_at: r.rotateAt.map(DateTime::from_timestamp_nanos),
            rotate_with: r.rotateWith,
        }).collect())
    }

    pub async fn update_key_info(
        &self,
        key: KeyId,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
    ) -> Result<()> {
        let name = name.as_ref();
        let desc = desc.as_ref();
        sqlx::query!(
            "UPDATE keys SET name = ?, description = ? WHERE id = ?",
            name,
            desc,
            key.0
        )
        .execute(&self.inner)
        .await
        .map_err(color_eyre::Report::from)
        .map(|_| ())
    }

    pub async fn update_key_secrets(
        &self,
        key: KeyId,
        secret: Option<Option<String>>,
        update_at: Option<Option<DateTime>>,
        update_with: Option<Option<String>>,
    ) -> Result<()> {
        if let Some(secret) = secret {
            sqlx::query!("UPDATE keys SET apiKey = ? WHERE id = ?", secret, key.0)
                .execute(&self.inner)
                .await
                .map_err(color_eyre::Report::from)?;
        }
        if let Some(update_at) = update_at {
            let update_at = update_at.map(|t| t.timestamp_nanos_opt().unwrap());
            sqlx::query!(
                "UPDATE keys SET rotateAt = ? WHERE id = ?",
                update_at,
                key.0
            )
            .execute(&self.inner)
            .await
            .map_err(color_eyre::Report::from)?;
        }
        if let Some(update_with) = update_with {
            sqlx::query!(
                "UPDATE keys SET rotateWith = ? WHERE id = ?",
                update_with,
                key.0
            )
            .execute(&self.inner)
            .await
            .map_err(color_eyre::Report::from)?;
        }

        Ok(())
    }

    pub async fn get_all_keys(&self) -> Result<Vec<TableKeys>> {
        sqlx::query!("SELECT * FROM keys")
            .fetch_all(&self.inner)
            .await
            .map_err(color_eyre::Report::from)
            .map(|v| {
                v.into_iter()
                    .map(|r| TableKeys {
                        id: KeyId(r.id),
                        name: r.name,
                        description: r.description,
                        key: r.apiKey,
                        rotate_at: r.rotateAt.map(DateTime::from_timestamp_nanos),
                        rotate_with: r.rotateWith,
                    })
                    .collect()
            })
    }
}
