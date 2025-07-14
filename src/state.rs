use std::sync::Arc;

use crate::config::Config;
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use color_eyre::Result;
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: crate::database::Database,
    pub config: Arc<crate::config::Config>,
    pub oauth2: Arc<
        openidconnect::core::CoreClient<
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointMaybeSet,
            EndpointMaybeSet,
        >,
    >,
    pub key: Key,
    pub http_client: openidconnect::reqwest::Client,
    pub template_env: minijinja::Environment<'static>,
}

impl AppState {
    const TEMPLATE_NAMES: &[&str] = &["template.html", "index.html"];

    pub async fn new(config: Config) -> Result<Self> {
        let config: Arc<Config> = Arc::new(config);
        let key = Key::try_from(config.cookie_secret.as_slice())?;
        let db = crate::database::Database::new(&config.db).await?;
        let http_client = openidconnect::reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");

        let client_metadata = openidconnect::core::CoreProviderMetadata::discover_async(
            openidconnect::IssuerUrl::from_url(config.oauth_issuer.clone()),
            &http_client,
        )
        .await?;
        let client = openidconnect::core::CoreClient::from_provider_metadata(
            client_metadata,
            openidconnect::ClientId::new(config.oauth_id.clone()),
            Some(openidconnect::ClientSecret::new(
                config.oauth_secret.clone(),
            )),
        )
        .set_redirect_uri(openidconnect::RedirectUrl::new(
            config.oauth_redirect.clone(),
        )?);

        let mut template_env = {
            let mut env = minijinja::Environment::new();
            env.set_loader(minijinja::path_loader(&config.template_dir));
            env
        };

        for t in Self::TEMPLATE_NAMES {
            let _ = template_env.get_template(t)?;
        }

        Ok(Self {
            db,
            config,
            key,
            oauth2: Arc::new(client),
            http_client,
            template_env,
        })
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(input: &AppState) -> Self {
        input.key.clone()
    }
}
