/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:49:03 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/26 14:35:54 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#![allow(unused)]

mod config;
mod database;
mod state;

use crate::{config::Config, state::AppState};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState::new(Config::from_env()?).await?;

    Ok(())
}
