use axum::{extract::State, response::Redirect};
use axum_extra::{
    extract::{CookieJar, PrivateCookieJar},
    routing::RouterExt,
};

use crate::state::AppState;

mod client_all;
mod client_key;
mod index;
mod key;

macro_rules! serve_file_handler {
    ($state:expr, $path:expr) => {{ tower_http::services::ServeFile::new(format!("{}/{}", $state.config.static_dir, $path)) }};
}

fn static_files_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route_service(
            "/favicon.ico",
            serve_file_handler!(state, "img/favicon.ico"),
        )
        .route_service(
            "/manifest.json",
            serve_file_handler!(state, "manifest.json"),
        )
        .route_service(
            "/service_worker.json",
            serve_file_handler!(state, "js/service_worker.json"),
        )
        .route_service(
            "/apple-touch-icon.png",
            serve_file_handler!(state, "img/apple-touch-icon.png"),
        )
}

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(index::get_index))
        .route_with_tsr("/keys", axum::routing::get(key::get_key))
        .route_with_tsr("/clients", axum::routing::get(client_all::get_all_clients))
        .route_with_tsr(
            "/client/{id}",
            axum::routing::get(client_key::get_client_key),
        )
        .with_state(state.clone())
        .merge(static_files_router(state.clone()))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct UserInfo {
    name: String,
    id: i64,
}

impl From<crate::database::users::TableUsers> for UserInfo {
    fn from(value: crate::database::users::TableUsers) -> Self {
        Self {
            name: value.name,
            id: value.id.inner(),
        }
    }
}
