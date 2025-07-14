use axum::{extract::State, http::StatusCode, response::Html};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::api::ErrorToStatusCode as _;

#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn get_index(
    user: crate::auth::UserAuthRedirect,
    State(state): State<crate::AppState>,
) -> Result<Html<String>, StatusCode> {
    info!("Rendering index.html template");
    let a = state.template_env.get_template("index.html").to_status()?;
    let user: super::UserInfo = user
        .get_user(&state.db)
        .await
        .to_status()?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .into();

    a.render(serde_json::json!({
        "self": user,
    }))
    .map(Html)
    .to_status()
}
