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
    // currently the key is in plaintext, when everything will work, the key will be encrypted in
    // the database using AES-GCM
    pub api_key: Vec<u8>,

    pub update_at: Option<DateTime>,
    // same as the `api_key`
    pub update_with: Option<Vec<u8>>,
}

impl Database {
    pub async fn create_key(
        &self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        key: String,
        update_at: Option<DateTime>,
        update_with: Option<String>,
    ) -> Result<KeyId> {
        let name = name.as_ref();
        let desc = desc.as_ref();

        let k = key.as_str();

        let u_at = update_at.map(|d| d.timestamp());
        let u_with = update_with.as_deref();

        let query = sqlx::query!(
            "INSERT INTO keys ('name', 'description', 'apiKey', 'updateAt', 'updateWith') VALUES (?, ?, ?, ?, ?) RETURNING id",
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
            api_key: s.apiKey.into_bytes(),
            update_at: s.updateAt.map(DateTime::from_timestamp_nanos),
            update_with: s.updateWith.map(|s| s.into_bytes()),
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
            api_key: r.apiKey.into_bytes(),
            update_at: r.updateAt.map(DateTime::from_timestamp_nanos),
            update_with: r.updateWith.map(String::into_bytes),
        }).collect())
    }
}
