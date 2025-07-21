use axum::{extract::State, http::StatusCode, response::Html};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::api::ErrorToStatusCode as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyInfo {
    id: i64,
    name: String,
    description: String,
    secret: String,
    rotate_at: Option<crate::database::Date>,
    rotate_with: String,
}

impl From<crate::database::keys::TableKeys> for KeyInfo {
    fn from(value: crate::database::keys::TableKeys) -> Self {
        Self {
            name: value.name,
            id: value.id.inner(),
            description: value.description,
            rotate_at: value.rotate_at,
            secret: value.key.unwrap_or_default(),
            rotate_with: value.rotate_with.unwrap_or_default(),
        }
    }
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn get_key(
    user: crate::auth::UserAuthRedirect,
    State(state): State<crate::AppState>,
) -> Result<Html<String>, StatusCode> {
    let a = state.template_env.get_template("keys.html").to_status()?;
    let user: super::UserInfo = user
        .get_user(&state.db)
        .await
        .to_status()?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .into();

    let all_keys = state
        .db
        .get_all_keys()
        .await
        .to_status()?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<KeyInfo>>();

    a.render(serde_json::json!({
        "self": user,
        "keys": all_keys,
    }))
    .map(Html)
    .to_status()
}
