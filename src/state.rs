/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   state.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:48:44 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/25 18:08:35 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::sync::Arc;

use crate::config::Config;
use color_eyre::Result;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: crate::database::Database,
    pub config: Arc<crate::config::Config>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        Ok(Self {
            db: crate::database::Database::new(&config.db).await?,
            config: config.into(),
        })
    }
}
