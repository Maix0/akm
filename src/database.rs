/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   database.rs                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:49:06 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/26 15:04:52 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use color_eyre::{Result, eyre::eyre};
use futures::StreamExt;
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use sha2::Digest;
use sqlx::Executor;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Database {
    inner: sqlx::SqlitePool,
}

type DateTime = chrono::DateTime<chrono::Utc>;

impl Database {
    const INIT_SCRIPT: &str = include_str!("./init.sql");

    pub async fn new(path: impl AsRef<str>) -> Result<Self> {
        let path = path.as_ref();
        let db = sqlx::SqlitePool::connect(path).await?;
        {
            let mut s = db.execute_many(Self::INIT_SCRIPT);
            while s.next().await.transpose()?.is_some() {}
        }
        Ok(Database { inner: db })
    }
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
}

impl Database {
    pub async fn create_clientkey(
        &self,
        client: ClientId,
        key: KeyId,
    ) -> Result<(ClientKeyId, SecretString)> {
        let sha = [0u8; 32];

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
            "INSERT INTO clients_key ('clientID', 'keyID', 'secret') VALUES (?, ?, ?) RETURNING id",
            client.0,
            key.0,
            tok,
        )
        .fetch_one(&self.inner)
        .await?;

        Ok((
            ClientKeyId(
                query
                    .id
                    .ok_or(eyre!("client<->key relation already exists"))?,
            ),
            token,
        ))
    }
    async fn fetch_client_key(&self, key: ClientKeyId) -> Result<Option<TableClientsKey>> {
        let query = sqlx::query!("SELECT * FROM clients_key where id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableClientsKey {
            id: ClientKeyId(s.id),
            client_id: ClientId(s.clientID),
            key_id: KeyId(s.keyID),
            secret: todo!(),
            last_used: s.lastUsed.map(DateTime::from_timestamp_nanos),
        }))
    }
}

impl Database {
    pub async fn create_key(
        &self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        key: SecretString,
        update_at: Option<DateTime>,
        update_with: Option<SecretString>,
    ) -> Result<KeyId> {
        let name = name.as_ref();
        let desc = desc.as_ref();

        let k = key.expose_secret();

        let u_at = update_at.map(|d| d.timestamp_nanos_opt().unwrap());
        let u_with = update_with.as_ref().map(|s| s.expose_secret());

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

    async fn fetch_key(&self, key: KeyId) -> Result<Option<TableKeys>> {
        let query = sqlx::query!("SELECT * FROM keys WHERE id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableKeys {
            id: KeyId(s.id),
            name: s.name,
            description: s.description,
            api_key: s.apiKey.into_bytes().into(),
            update_at: s.updateAt.map(DateTime::from_timestamp_nanos),
            update_with: s.updateWith.map(|s| s.into_bytes().into()),
        }))
    }
}

impl Database {
    pub async fn create_client(
        &self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
    ) -> Result<ClientId> {
        let name = name.as_ref();
        let desc = desc.as_ref();

        let query = sqlx::query!(
            "INSERT INTO clients ('name', 'description') VALUES (?, ?) RETURNING id",
            name,
            desc,
        )
        .fetch_one(&self.inner)
        .await?;
        Ok(ClientId(query.id))
    }
    async fn fetch_client(&self, key: ClientId) -> Result<Option<TableClients>> {
        let query = sqlx::query!("SELECT * FROM clients where id = ? LIMIT 1", key.0)
            .fetch_optional(&self.inner)
            .await?;

        Ok(query.map(|mut s| TableClients {
            id: ClientId(s.id),
            name: s.name,
            description: s.description,
        }))
    }
}

macro_rules! defineID {
    ($($name:ident),*$(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $name(i64);
            impl $name {
                pub fn inner(self) -> i64 { self.0 }
            }
        )*
    };
}

defineID!(ClientId, UserId, KeyId, ClientKeyId);

#[derive(Debug, Clone)]
pub struct TableClients {
    id: ClientId,
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
pub struct TableClientsKey {
    id: ClientKeyId,
    client_id: ClientId,
    key_id: KeyId,
    secret: SecretString,
    last_used: Option<DateTime>,
}

#[derive(Debug, Clone)]
pub struct TableKeys {
    id: KeyId,
    name: String,
    description: String,
    // currently the key is in plaintext, when everything will work, the key will be encrypted in
    // the database using AES-GCM
    api_key: SecretSlice<u8>,

    update_at: Option<DateTime>,
    // same as the `api_key`
    update_with: Option<SecretSlice<u8>>,
}

#[derive(Debug, Clone)]
pub struct TableUsers {
    id: UserId,
    name: String,
    token: SecretString,
}
