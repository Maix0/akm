use axum::{extract::State, http::StatusCode, response::Html};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::api::ErrorToStatusCode as _;

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
pub async fn get_all_clients(
    user: crate::auth::UserAuthRedirect,
    State(state): State<crate::AppState>,
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

    let all_clients = state
        .db
        .get_all_clients()
        .await
        .to_status()?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ClientInfo>>();

    a.render(serde_json::json!({
        "self": user,
        "clients": all_clients,
    }))
    .map(Html)
    .to_status()
}
