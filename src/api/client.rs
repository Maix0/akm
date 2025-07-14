use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::DateTime;
use serde::de::DeserializeOwned;

use super::ErrorToStatusCode;
use crate::{
    database::{clients::ClientId, keys::KeyId},
    state::AppState,
};

#[derive(serde::Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct ClientInfo {
    pub id: i64,
    pub desc: String,
    pub name: String,
}

impl From<crate::database::clients::TableClients> for ClientInfo {
    fn from(v: crate::database::clients::TableClients) -> Self {
        Self {
            id: v.id.inner(),
            name: v.name,
            desc: v.description,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone, Debug)]
pub struct ClientInfoNoId {
    pub name: String,
    pub desc: String,
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(post, path = "/client/new", 
    responses(
        (status = OK, body = i64, description = "new Client Created"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    request_body(content = inline(ClientInfoNoId), content_type = "application/json")
)]
pub async fn client_new(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Json(new_info): Json<ClientInfoNoId>,
) -> Result<Json<i64>, StatusCode> {
    let AppState { ref db, .. } = state;

    db.create_client(new_info.name.as_str(), new_info.desc.as_str())
        .await
        .to_status()
        .map(ClientId::inner)
        .map(Json)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(get, path = "/client/{client}/", 
    responses(
        (status = OK, body = inline(ClientInfoNoId), description = "Info of a client)"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
        (status = NOT_FOUND, description = "The client doesn't exist"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
    ))
]
pub async fn client_info(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path(client): Path<i64>,
) -> Result<Json<ClientInfo>, StatusCode> {
    let AppState { ref db, .. } = state;

    super::utils::client_from_raw(db, client)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)
        .map(ClientInfo::from)
        .map(Json)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(put, path = "/client/{client}/", 
    responses(
        (status = OK, description = "Info of a client)"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
        (status = NOT_FOUND, description = "The client doesn't exist"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
    ),
    request_body(content = inline(ClientInfoNoId), content_type = "application/json")
    )
]
pub async fn client_set_info(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path(client): Path<i64>,
    Json(info): Json<ClientInfoNoId>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;

    let client = super::utils::client_from_raw(db, client)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.update_client_info(client.id, info.name, info.desc)
        .await
        .to_status()
        .map(|_| StatusCode::OK)
}

#[derive(serde::Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct KeyInfo {
    id: i64,
    name: String,
    desc: String,
    update_at: Option<i64>,
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(get, path = "/client/{client}/key/list", 
    responses(
        (status = OK, body = inline(Vec<KeyInfo>), description = "Info of a client"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
        (status = NOT_FOUND, description = "The client doesn't exist"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
    ))
]
pub async fn client_list_keys(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path(client): Path<i64>,
) -> Result<Json<Vec<KeyInfo>>, StatusCode> {
    let AppState { ref db, .. } = state;

    let client = super::utils::client_from_raw(db, client)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.get_all_keys_from_client(client.id)
        .await
        .to_status()
        .map(|v| {
            v.into_iter().map(|s| KeyInfo {
                id: s.id.inner(),
                name: s.name,
                desc: s.description,
                update_at: s.rotate_at.as_ref().and_then(DateTime::timestamp_nanos_opt),
            })
        })
        .map(Iterator::collect)
        .map(Json)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(delete, path = "/client/{client}/delete", 
    responses(
        (status = OK, body = i64, description = "new Client Created"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
    )
)]
pub async fn client_delete(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((client,)): Path<(i64,)>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;
    let client = super::utils::client_from_raw(db, client)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    if db.remove_client(client.id).await.to_status()? {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(delete, path = "/client/{client}/key/{key}/delete", 
    responses(
        (status = OK, body = i64, description = "new Client Created"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
        ("key" = i64, Path, description = "The keyId"),
    )
)]
pub async fn client_delete_key(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((client, key)): Path<(i64, i64)>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;
    let k = super::utils::clientkey_from_client_and_key(db, client, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    if db.remove_clientkey(k.id).await.to_status()? {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(get, path = "/client/{client}/key/{key}/secret", 
    responses(
        (status = OK, body = String, description = "Secret to actually get the key"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
        (status = NOT_FOUND, description = "The client or key (or the key<->client relation) doesn't exist "),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
        ("key" = i64, Path, description = "The keyId"),
    ))
]
pub async fn client_get_secret(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((client, key)): Path<(i64, i64)>,
) -> Result<String, StatusCode> {
    let AppState { ref db, .. } = state;

    super::utils::clientkey_from_client_and_key(db, client, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)
        .map(|v| v.secret)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(put, path = "/client/{client}/key/{key}/new_secret", 
    responses(
        (status = OK, body = String, description = "Secret was updated to the value that has been returned"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
        (status = NOT_FOUND, description = "The client or the key associated with the client doesn't exist"),
    ),
    params(
        ("client" = i64, Path, description = "The client"),
        ("key" = i64, Path, description = "The key"),
    ))
]
pub async fn client_new_secret(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((client, key)): Path<(i64, i64)>,
) -> Result<String, StatusCode> {
    let AppState { ref db, .. } = state;
    let client_key = super::utils::clientkey_from_client_and_key(db, client, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.update_client_secret(client_key.id)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)
}
