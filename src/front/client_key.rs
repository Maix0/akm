use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::api::ErrorToStatusCode as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyInfo {
    id: i64,
    k_name: String,
    k_desc: String,
    secret: String,
    last_used: Option<crate::database::Date>,
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
        .get_template("clients.html")
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
    let mut own_keys_info: Vec<KeyInfo> = Vec::new();

    for k in all_clients_keys {
        //let k_associated = state
        //    .db
        //    .fetch_client_key_with_client(client_id, k.id)
        //    .await
        //    .to_status()?;
    }

    let client: ClientInfo = client.into();
    a.render(serde_json::json!({
        "self": user,
        "own_keys": own_keys_info,
        "client": client,
    }))
    .map(Html)
    .to_status()
}
