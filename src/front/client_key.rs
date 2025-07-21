use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::{api::ErrorToStatusCode as _, database::keys::KeyId};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssocKeyInfo {
    id: i64,
    k_id: i64,
    k_name: String,
    k_desc: String,
    secret: String,
    last_used: Option<crate::database::Date>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyInfo {
    id: i64,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientInfo {
    id: i64,
    name: String,
    description: String,
}

impl From<crate::database::clients::TableClients> for ClientInfo {
    fn from(value: crate::database::clients::TableClients) -> Self {
        Self {
            id: value.id.inner(),
            name: value.name,
            description: value.description,
        }
    }
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn get_client_key(
    user: crate::auth::UserAuthRedirect,
    State(state): State<crate::AppState>,
    Path((cid,)): Path<(i64,)>,
) -> Result<Html<String>, StatusCode> {
    let a = state
        .template_env
        .get_template("client_key.html")
        .to_status()?;
    let user: super::UserInfo = user
        .get_user(&state.db)
        .await
        .to_status()?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .into();
    let client = crate::api::utils::client_from_raw(&state.db, cid)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;
    let client_id = client.id;

    let all_clients_keys = state
        .db
        .get_all_keys_from_client(client.id)
        .await
        .to_status()?;
    let mut own_keys_info: Vec<AssocKeyInfo> = Vec::new();
    let mut own_keys_id: HashSet<i64> = HashSet::new();

    for k in all_clients_keys {
        let Some(k_associated) = state
            .db
            .fetch_client_key_from_client_and_key(client_id, k.id)
            .await
            .to_status()?
        else {
            tracing::warn!(
                "ClientKey not found for {}:{}",
                client_id.inner(),
                k.id.inner()
            );
            continue;
        };
        own_keys_id.insert(k.id.inner());
        own_keys_info.push(AssocKeyInfo {
            id: k_associated.id.inner(),
            k_name: k.name,
            k_desc: k.description,
            k_id: k.id.inner(),
            secret: k_associated.secret,
            last_used: k_associated.last_used,
        })
    }
    let not_own_keys = state
        .db
        .get_all_keys()
        .await
        .to_status()?
        .into_iter()
        .map(|k| KeyInfo {
            id: k.id.inner(),
            name: k.name,
        })
        .filter(|k| !own_keys_id.contains(&k.id))
        .collect::<Vec<_>>();

    let client: ClientInfo = client.into();
    a.render(serde_json::json!({
        "self": user,
        "not_own_keys": not_own_keys,
        "own_keys": own_keys_info,
        "client": client,
    }))
    .map(Html)
    .to_status()
}
