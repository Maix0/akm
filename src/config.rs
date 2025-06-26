/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   config.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/24 17:42:24 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/25 18:07:25 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use color_eyre::{Result, eyre::eyre};
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub cookie_secret: String,
    pub db: String,
    pub port: u16,
    pub ip: Ipv4Addr,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            cookie_secret: std::env::var("SECRET")?,
            db: std::env::var("DATABASE")?,
            port: std::env::var("PORT")?.parse()?,
            ip: std::env::var("IP")?.parse()?,
        })
    }
}
