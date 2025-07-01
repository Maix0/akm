use color_eyre::Result;

use crate::database::{clientkeys::ClientKeyId, clients::ClientId, keys::KeyId, users::UserId};

pub async fn client_from_raw(
    db: &crate::database::Database,
    client: i64,
) -> Result<Option<crate::database::clients::TableClients>> {
    let Some(client) = ClientId::from_raw(db, client).await? else {
        return Ok(None);
    };
    db.fetch_client(client).await
}

pub async fn user_from_raw(
    db: &crate::database::Database,
    user: i64,
) -> Result<Option<crate::database::users::TableUsers>> {
    let Some(user) = UserId::from_raw(db, user).await? else {
        return Ok(None);
    };
    db.fetch_user(user).await
}

pub async fn key_from_raw(
    db: &crate::database::Database,
    key: i64,
) -> Result<Option<crate::database::keys::TableKeys>> {
    let Some(key) = KeyId::from_raw(db, key).await? else {
        return Ok(None);
    };
    db.fetch_key(key).await
}

pub async fn clientkey_from_raw(
    db: &crate::database::Database,
    clientkey: i64,
) -> Result<Option<crate::database::clientkeys::TableClientsKey>> {
    let Some(client_key) = ClientKeyId::from_raw(db, clientkey).await? else {
        return Ok(None);
    };
    db.fetch_client_key(client_key).await
}

pub async fn clientkey_from_client_and_key(
    db: &crate::database::Database,
    client: i64,
    key: i64,
) -> Result<Option<crate::database::clientkeys::TableClientsKey>> {
    let Some(client) = ClientId::from_raw(db, client).await? else {
        return Ok(None);
    };
    let Some(key) = KeyId::from_raw(db, key).await? else {
        return Ok(None);
    };
    db.fetch_client_key_from_client_and_key(client, key).await
}
