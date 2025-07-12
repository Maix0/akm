#![allow(unused)]

mod api;
mod auth;
mod config;
mod database;
mod state;

use crate::{config::Config, state::AppState};
use color_eyre::Result;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::OpenApi)]
#[openapi()]
struct Api;

pub fn router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(api::client::client_delete))
        .routes(routes!(api::client::client_delete_key,))
        .routes(routes!(api::client::client_get_secret,))
        .routes(routes!(api::client::client_info,))
        .routes(routes!(api::client::client_list_keys,))
        .routes(routes!(api::client::client_new,))
        .routes(routes!(api::client::client_new_secret))
        .routes(routes!(api::key::key_new))
        .routes(routes!(api::key::key_info, api::key::key_set_info))
        .routes(routes!(api::key::key_delete))
        .routes(routes!(api::key::key_rotate))
        .routes(routes!(api::key::key_update_secret, api::key::key_secret))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init()?;
    sqlx::any::install_default_drivers();
    let state = AppState::new(Config::from_env()?).await?;

    let (router, api) = OpenApiRouter::with_openapi(Api::openapi())
        .nest("/api/", router(state.clone()))
        .split_for_parts();

    let router: axum::Router<()> = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .nest("/auth", auth::router(state.clone()));

    let socket = TcpListener::bind((state.config.ip, state.config.port)).await?;
    axum::serve(socket, router).await?;
    Ok(())
}
