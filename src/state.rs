/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   state.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:48:44 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/30 14:25:23 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::sync::Arc;

use crate::config::Config;
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use color_eyre::Result;
use secrecy::ExposeSecret;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: crate::database::Database,
    pub config: Arc<crate::config::Config>,
    pub key: Key,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let config: Arc<Config> = Arc::new(config);
        let key = Key::try_from(config.cookie_secret.expose_secret())?;
        let db = crate::database::Database::new(&config.db).await?;

        Ok(Self { db, config, key })
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(input: &AppState) -> Self {
        input.key.clone()
    }
}
