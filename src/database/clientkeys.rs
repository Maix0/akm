use crate::database::keys::KeyId;

use super::Database;
use super::DateTime;

use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;
use std::time::SystemTime;

super::defineID!(ClientKeyId => "clients_key");

impl Database {
    pub async fn create_clientkey(
        &self,
        client: super::clients::ClientId,
        key: super::keys::KeyId,
    ) -> Result<(ClientKeyId, SecretString)> {
        let sha = {
            let mut sha = [0u8; 32];
            let mut rng = rand::rng();
            rng.fill_bytes(&mut sha);
            sha
        };

        let token = SecretString::from({
            let mut s = String::with_capacity(sha.len() * 2);
            for &x in sha.as_slice() {
                use std::fmt::Write;
                write!(s, "{x:02x}").unwrap();
            }
            s
        });

        let tok = token.expose_secret();
        let query = sqlx::query!(
            "INSERT INTO clients_key ('clientID', 'keyID', 'secret') VALUES (?, ?, ?) RETURNING id, secret",
            client.0,
            key.0,
            tok,
        )
        .fetch_one(&self.inner)
        .await?;

        Ok((ClientKeyId(query.id), query.secret.into()))
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

        Ok(query.map(|mut s| TableClientsKey {
            id: ClientKeyId(s.id),
            client_id: super::clients::ClientId(s.clientID),
            key_id: super::keys::KeyId(s.keyID),
            secret: todo!(),
            last_used: s.lastUsed.map(|t| DateTime::from_timestamp(t, 0).unwrap()),
        }))
    }

    pub async fn fetch_client_key(&self, key: ClientKeyId) -> Result<Option<TableClientsKey>> {
        let query = sqlx::query!("SELECT * FROM clients_key where id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableClientsKey {
            id: ClientKeyId(s.id),
            client_id: super::clients::ClientId(s.clientID),
            key_id: super::keys::KeyId(s.keyID),
            secret: todo!(),
            last_used: s.lastUsed.map(|t| DateTime::from_timestamp(t, 0).unwrap()),
        }))
    }

    pub async fn get_client_key_from_secret(
        &self,
        secret: SecretString,
    ) -> Result<Option<TableClientsKey>> {
        let s = secret.expose_secret();
        let query = sqlx::query!("SELECT * FROM clients_key where secret = ? LIMIT 1", s)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableClientsKey {
            id: ClientKeyId(s.id),
            client_id: super::clients::ClientId(s.clientID),
            key_id: super::keys::KeyId(s.keyID),
            secret: todo!(),
            last_used: s.lastUsed.map(|t| DateTime::from_timestamp(t, 0).unwrap()),
        }))
    }

    // return true if the client_key has been updated
    pub async fn update_client_key_last_used(&self, key: ClientKeyId) -> Result<bool> {
        let date = DateTime::from(SystemTime::now());
        let timestamp = date.timestamp();

        sqlx::query!(
            "UPDATE clients_key SET lastUsed = ? WHERE id = ?",
            timestamp,
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
    pub async fn update_client_secret(&self, key: ClientKeyId) -> Result<Option<SecretString>> {
        let sha = {
            let mut sha = [0u8; 32];
            let mut rng = rand::rng();
            rng.fill_bytes(&mut sha);
            sha
        };

        let token = SecretString::from({
            let mut s = String::with_capacity(sha.len() * 2);
            for &x in sha.as_slice() {
                use std::fmt::Write;
                write!(s, "{x:02x}").unwrap();
            }
            s
        });
        let s = token.expose_secret();

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
    pub secret: SecretString,
    pub last_used: Option<DateTime>,
}
