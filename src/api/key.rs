use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::error;

use crate::database::Date;
use crate::{api::ErrorToStatusCode, state::AppState};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct KeyInfo {
    pub name: String,
    pub desc: String,
    pub has_key: bool,
    pub rotate_at: Option<Date>,
    pub has_rotate_key: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct NewKeyInfo {
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct KeySetSecrets {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::utils::double_option"
    )]
    secret: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::utils::double_option"
    )]
    rotate_at: Option<Option<Date>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::utils::double_option"
    )]
    rotate_with: Option<Option<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct KeyGetSecrets {
    secret: Option<String>,
    rotate_at: Option<Date>,
    rotate_with: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct KeyInfoUpdate {
    pub name: String,
    pub desc: String,
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(get, path = "/key/{key}/", 
    responses(
        (status = OK, body = inline(KeyInfo), description = "Key information (no secrets)"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("key" = i64, Path, description = "The key"),
    ),
)]
pub async fn key_info(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
) -> Result<Json<KeyInfo>, StatusCode> {
    let AppState { ref db, .. } = state;

    super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)
        .map(|key| KeyInfo {
            desc: key.description,
            has_key: key.key.is_some(),
            has_rotate_key: key.rotate_with.is_some(),
            name: key.name,
            rotate_at: key.rotate_at,
        })
        .map(Json)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(put, path = "/key/{key}/", 
    responses(
        (status = OK, description = "Key information updated (no secrets nor time to rotate)"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    request_body(content = inline(KeyInfoUpdate), content_type = "application/json")
)]
pub async fn key_set_info(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
    Json(update): Json<KeyInfoUpdate>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;

    let key = super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.update_key_info(key.id, update.name, update.desc)
        .await
        .to_status()
        .map(|_| StatusCode::OK)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(delete, path = "/key/{key}/delete", 
    responses(
        (status = OK, description = "Key deleted"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("key" = i64, Path, description = "The key"),
    ),
)]
pub async fn key_delete(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;

    let key = super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.delete_all_with_key_id(key.id).await.to_status()?;
    db.remove_key(key.id).await.to_status()?;
    Ok(StatusCode::OK)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(put, path = "/key/{key}/rotate", 
    responses(
        (status = OK, description = "Key Rotated"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("key" = i64, Path, description = "The key"),
    ),
)]
pub async fn key_rotate(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;

    let key = super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;
    dbg!(&key);

    db.update_key_secrets(key.id, Some(key.rotate_with), Some(None), Some(None))
        .await
        .to_status()
        .map(|_| StatusCode::OK)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(put, path = "/key/{key}/secret", 
    responses(
        (status = OK, description = "Key Rotated"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("key" = i64, Path, description = "The key"),
    ),
    request_body(content = inline(KeySetSecrets), content_type = "application/json")
)]
pub async fn key_update_secret(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
    Json(update): Json<KeySetSecrets>,
) -> Result<StatusCode, StatusCode> {
    let AppState { ref db, .. } = state;

    let key = super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)?;

    db.update_key_secrets(key.id, update.secret, update.rotate_at, update.rotate_with)
        .await
        .to_status()
        .map(|_| StatusCode::OK)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(get, path = "/key/{key}/secret", 
    responses(
        (status = OK, body = inline(KeyGetSecrets), description = "Key secrets"),
        (status = NOT_FOUND, description = "Key not found"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    params(
        ("key" = i64, Path, description = "The key"),
    ),
)]
pub async fn key_secret(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Path((key,)): Path<(i64,)>,
) -> Result<Json<KeyGetSecrets>, StatusCode> {
    let AppState { ref db, .. } = state;

    super::utils::key_from_raw(db, key)
        .await
        .to_status()?
        .ok_or(StatusCode::NOT_FOUND)
        .map(|key| KeyGetSecrets {
            secret: key.key,
            rotate_at: key.rotate_at,
            rotate_with: key.rotate_with,
        })
        .map(Json)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
#[utoipa::path(post, path = "/key/new", 
    responses(
        (status = OK, body = i64, description = "Key was created"),
        (status = BAD_REQUEST, description = "Invalid Request: name must be alphanumeric or `-`/`_`. description must be between 0 and 1024 characters"),
        (status = FORBIDDEN, description = "Invalid Auth cookie"),
    ),
    request_body(content = inline(NewKeyInfo), content_type = "application/json")
)]
pub async fn key_new(
    _: crate::auth::UserAuth,
    State(state): State<crate::AppState>,
    Json(info): Json<NewKeyInfo>,
) -> Result<Json<i64>, StatusCode> {
    let AppState { ref db, .. } = state;

    if !(info
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
    {
        error!(
            "new key name isn't only alphanumeric or `_`/`-`: {}",
            info.name
        );
        return Err(StatusCode::BAD_REQUEST);
    }
    if !(0..=1024).contains(&info.desc.chars().count()) {
        error!("new key description is too long");
        return Err(StatusCode::BAD_REQUEST);
    }

    db.create_key(info.name, info.desc, None, None, None)
        .await
        .to_status()
        .map(|k| k.inner())
        .map(Json)
}
