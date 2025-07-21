use crate::database::keys::KeyId;

use super::Database;
use super::Date;

use chrono::DateTime;
use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use rand::RngCore;
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

super::defineID!(ClientKeyId => "clients_key");

impl Database {
    pub async fn create_clientkey(
        &self,
        client: super::clients::ClientId,
        key: super::keys::KeyId,
    ) -> Result<(ClientKeyId, String)> {
        let sha = {
            let mut sha = [0u8; 32];
            let mut rng = rand::rng();
            rng.fill_bytes(&mut sha);
            sha
        };

        let token = {
            let mut s = String::with_capacity(sha.len() * 2);
            for &x in sha.as_slice() {
                use std::fmt::Write;
                write!(s, "{x:02x}").unwrap();
            }
            s
        };

        let tok = token.as_str();
        let query = sqlx::query!(
            "INSERT INTO clients_key ('clientID', 'keyID', 'secret') VALUES (?, ?, ?) RETURNING id, secret",
            client.0,
            key.0,
            tok,
        )
        .fetch_one(&self.inner)
        .await?;

        Ok((ClientKeyId(query.id), query.secret))
    }

    pub async fn remove_clientkey(&self, key: ClientKeyId) -> Result<bool> {
        sqlx::query!("DELETE FROM clients_key WHERE id = ?", key.0)
            .execute(&self.inner)
            .await
            .inspect(|s| assert!(s.rows_affected() <= 1, "mutliple key with the same id"))
            .map(|s| s.rows_affected() == 1)
            .map_err(color_eyre::Report::from)
    }

    pub async fn fetch_client_key_from_client_and_key(
        &self,
        client: super::clients::ClientId,
        key: KeyId,
    ) -> Result<Option<TableClientsKey>> {
        let query = sqlx::query!(
            "SELECT * FROM clients_key WHERE clientID = ? AND keyID = ? LIMIT 1",
            client.0,
            key.0
        )
        .fetch_optional(&self.inner)
        .await?;

        query
            .map(|mut s| {
                Ok(TableClientsKey {
                    id: ClientKeyId(s.id),
                    client_id: super::clients::ClientId(s.clientID),
                    key_id: super::keys::KeyId(s.keyID),
                    secret: todo!(),
                    last_used: s.lastUsed.map(|t| Date::from_str(t.as_str())).transpose()?,
                })
            })
            .transpose()
    }

    pub async fn fetch_client_key(&self, key: ClientKeyId) -> Result<Option<TableClientsKey>> {
        let query = sqlx::query!("SELECT * FROM clients_key where id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        query
            .map(|mut s| {
                Ok(TableClientsKey {
                    id: ClientKeyId(s.id),
                    client_id: super::clients::ClientId(s.clientID),
                    key_id: super::keys::KeyId(s.keyID),
                    secret: todo!(),
                    last_used: s.lastUsed.map(|t| Date::from_str(t.as_str())).transpose()?,
                })
            })
            .transpose()
    }

    pub async fn get_client_key_from_secret(
        &self,
        secret: impl AsRef<str>,
    ) -> Result<Option<TableClientsKey>> {
        let s = secret.as_ref();
        let query = sqlx::query!("SELECT * FROM clients_key where secret = ? LIMIT 1", s)
            .fetch_optional(&self.inner)
            .await?;

        query
            .map(|mut s| {
                Ok(TableClientsKey {
                    id: ClientKeyId(s.id),
                    client_id: super::clients::ClientId(s.clientID),
                    key_id: super::keys::KeyId(s.keyID),
                    secret: todo!(),
                    last_used: s.lastUsed.map(|t| Date::from_str(t.as_str())).transpose()?,
                })
            })
            .transpose()
    }

    // return true if the client_key has been updated
    pub async fn update_client_key_last_used(&self, key: ClientKeyId) -> Result<bool> {
        let date = DateTime::<chrono::Utc>::from(SystemTime::now());
        let date = Date(date.date_naive());
        let date = date.to_string();

        sqlx::query!(
            "UPDATE clients_key SET lastUsed = ? WHERE id = ?",
            date,
            key.0
        )
        .execute(&self.inner)
        .await
        .inspect(|s| {
            assert!(
                s.rows_affected() <= 1,
                "multiple client_key share the same id"
            )
        })
        .map(|s| s.rows_affected() == 1)
        .map_err(color_eyre::Report::from)
    }

    // update the secret used by the client_key
    pub async fn update_client_secret(&self, key: ClientKeyId) -> Result<Option<String>> {
        let sha = {
            let mut sha = [0u8; 32];
            let mut rng = rand::rng();
            rng.fill_bytes(&mut sha);
            sha
        };

        let token = {
            let mut s = String::with_capacity(sha.len() * 2);
            for &x in sha.as_slice() {
                use std::fmt::Write;
                write!(s, "{x:02x}").unwrap();
            }
            s
        };
        let s = token.as_str();

        sqlx::query!("UPDATE clients_key SET secret = ? WHERE id = ?", s, key.0)
            .execute(&self.inner)
            .await
            .inspect(|s| {
                assert!(
                    s.rows_affected() <= 1,
                    "multiple client_key share the same id"
                )
            })
            .map(|s| s.rows_affected() == 1)
            .map_err(color_eyre::Report::from)
            .map(|c| c.then_some(token))
    }

    pub async fn delete_all_with_key_id(&self, key: super::keys::KeyId) -> Result<u64> {
        sqlx::query!("DELETE FROM clients_key WHERE keyID = ?", key.0)
            .execute(&self.inner)
            .await
            .map(|s| s.rows_affected())
            .map_err(color_eyre::Report::from)
    }

    pub async fn delete_all_with_client_id(&self, client: super::clients::ClientId) -> Result<u64> {
        sqlx::query!("DELETE FROM clients_key WHERE clientID = ?", client.0)
            .execute(&self.inner)
            .await
            .map(|s| s.rows_affected())
            .map_err(color_eyre::Report::from)
    }
}

#[derive(Debug, Clone)]
pub struct TableClientsKey {
    pub id: ClientKeyId,
    pub client_id: super::clients::ClientId,
    pub key_id: super::keys::KeyId,
    pub secret: String,
    pub last_used: Option<Date>,
}
