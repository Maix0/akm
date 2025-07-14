use base64::Engine;
use color_eyre::{Result, eyre::eyre};
use std::{ffi::OsStr, net::Ipv4Addr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub cookie_secret: Vec<u8>,
    pub db: String,
    pub port: u16,
    pub ip: Ipv4Addr,

    pub oauth_issuer: url::Url,
    pub oauth_secret: String,
    pub oauth_redirect: String,
    pub oauth_id: String,

    pub template_dir: String,
    pub static_dir: String,
}

fn get_var(k: impl AsRef<str>) -> color_eyre::Result<String> {
    let k = k.as_ref();

    std::env::var(k)
        .map_err(color_eyre::Report::from)
        .map_err(|e| e.wrap_err(format!("name: \"{k}\"")))
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            cookie_secret: {
                let s = get_var("COOKIE_SECRET")?;

                base64::engine::general_purpose::STANDARD.decode(&s)?
            },
            db: get_var("DATABASE")?,
            port: get_var("PORT")?.parse()?,
            ip: get_var("IP")?.parse()?,

            oauth_id: get_var("OAUTH2_ID")?,
            oauth_redirect: get_var("OAUTH2_REDIRECT")?,
            oauth_secret: get_var("OAUTH2_SECRET")?,
            oauth_issuer: get_var("OAUTH2_ISSUER")?.parse()?,

            template_dir: get_var("TEMPLATE_DIR")?,
            static_dir: get_var("STATIC_DIR")?,
        })
    }
}
