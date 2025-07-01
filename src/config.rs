use base64::Engine;
use color_eyre::{Result, eyre::eyre};
use secrecy::SecretSlice;
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub cookie_secret: SecretSlice<u8>,
    pub db: String,
    pub port: u16,
    pub ip: Ipv4Addr,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            cookie_secret: {
                let s = std::env::var("SECRET")?;
                let v = base64::engine::general_purpose::STANDARD.decode(&s)?;
                v.into()
            },
            db: std::env::var("DATABASE")?,
            port: std::env::var("PORT")?.parse()?,
            ip: std::env::var("IP")?.parse()?,
        })
    }
}
