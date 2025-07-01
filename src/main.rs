#![allow(unused)]

mod api;
mod auth;
mod config;
mod database;
mod state;

use crate::{config::Config, state::AppState};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init()?;
    sqlx::any::install_default_drivers();
    let state = AppState::new(Config::from_env()?).await?;

    Ok(())
}
