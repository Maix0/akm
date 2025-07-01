use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use secrecy::{ExposeSecret, SecretString};

use super::ErrorToStatusCode;
use crate::{
    database::{clients::ClientId, keys::KeyId},
    state::AppState,
};

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
        .map(|s| s.expose_secret().to_string())
}
