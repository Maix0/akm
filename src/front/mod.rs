use axum::{extract::State, response::Redirect};
use axum_extra::extract::{CookieJar, PrivateCookieJar};

use crate::state::AppState;

mod index;

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
        .with_state(state.clone())
        .merge(static_files_router(state.clone()))
}
