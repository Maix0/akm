/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:49:03 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/30 16:14:14 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#![allow(unused)]

mod auth;
mod config;
mod database;
mod routes;
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
