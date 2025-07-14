use std::{borrow::Cow, collections::HashMap};

use axum::{
    Router,
    extract::{FromRequestParts, OptionalFromRequestParts, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, Key, SameSite},
};
use color_eyre::eyre::{self, ContextCompat as _};
use openidconnect::{
    AuthorizationCode, CsrfToken, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, UserInfoClaims,
};
use tracing::{debug, error, warn};

use crate::{
    database::users::{TableUsers, UserId},
    state::AppState,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct UserAuth(crate::database::users::UserId);
pub struct UserAuthRedirect(crate::database::users::UserId);

const AUTH_COOKIE: &str = "session";

impl FromRequestParts<AppState> for UserAuth {
    type Rejection = (PrivateCookieJar, StatusCode);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .unwrap();
        let Some(c) = cookies.get(AUTH_COOKIE) else {
            return Err((cookies.remove(AUTH_COOKIE), StatusCode::FORBIDDEN));
        };

        match state.db.get_user_from_token(c.value().into()).await {
            Err(e) => {
                error!("Failed to get user from db: {e}");
                Err((cookies, StatusCode::INTERNAL_SERVER_ERROR))
            }
            Ok(None) => {
                debug!("Cookie removed");
                Err((cookies.remove(AUTH_COOKIE), StatusCode::FORBIDDEN))
            }
            Ok(Some(v)) => {
                debug!("User is logged");
                Ok(Self(v))
            }
        }
    }
}

impl OptionalFromRequestParts<AppState> for UserAuth {
    type Rejection = (PrivateCookieJar, StatusCode);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <Self as FromRequestParts<AppState>>::from_request_parts(parts, state).await {
            Ok(v) => Ok(Some(v)),
            Err((_, StatusCode::FORBIDDEN)) => Ok(None),
            Err((c, s)) => Err((c, s)),
        }
    }
}

impl FromRequestParts<AppState> for UserAuthRedirect {
    type Rejection = (PrivateCookieJar, Redirect);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        <UserAuth as FromRequestParts<AppState>>::from_request_parts(parts, state)
            .await
            .map(|UserAuth(id)| UserAuthRedirect(id))
            .map_err(|(c, _)| (c, Redirect::to("/auth/login")))
    }
}

impl OptionalFromRequestParts<AppState> for UserAuthRedirect {
    type Rejection = (PrivateCookieJar, Redirect);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        <UserAuth as OptionalFromRequestParts<AppState>>::from_request_parts(parts, state)
            .await
            .map(|o| o.map(|UserAuth(id)| UserAuthRedirect(id)))
            .map_err(|(c, _)| (c, Redirect::to("/auth/login")))
    }
}

impl UserAuthRedirect {
    pub async fn get_user(
        self,
        db: &crate::database::Database,
    ) -> color_eyre::Result<Option<TableUsers>> {
        db.fetch_user(self.0).await
    }

    pub async fn get_id(self) -> UserId {
        self.0
    }
}

impl UserAuth {
    pub async fn get_user(
        self,
        db: &crate::database::Database,
    ) -> color_eyre::Result<Option<TableUsers>> {
        db.fetch_user(self.0).await
    }

    pub async fn get_id(self) -> UserId {
        self.0
    }
}

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/logout", get(logout))
        .route("/login", get(oauth2_login))
        .route("/callback", get(oauth2_callback))
        .with_state(state)
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
async fn oauth2_login(
    State(state): State<crate::state::AppState>,
    hmap: HeaderMap,
    query: Query<HashMap<String, Option<String>>>,
) -> Result<(PrivateCookieJar, Redirect), StatusCode> {
    let jar = PrivateCookieJar::from_headers(&hmap, state.key.clone());
    let (challenge, result) = PkceCodeChallenge::new_random_sha256();

    let (url, _csrf, _nonce) = state
        .oauth2
        .authorize_url(
            openidconnect::AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(challenge)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_redirect_uri(Cow::Owned(
            RedirectUrl::new(state.config.oauth_redirect.clone())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        ))
        .url();

    Ok((
        jar.add(Cookie::new("pkce", result.secret().clone())),
        Redirect::to(url.as_str()),
    ))
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
async fn oauth2_callback(
    State(state): State<crate::AppState>,
    Query(params): Query<HashMap<String, String>>,
    hmap: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let jar = PrivateCookieJar::from_headers(&hmap, state.key.clone());
    let inner = || async {
        let Some(code): Option<&String> = params.get("code") else {
            warn!("oauth2 callback no code querystring");
            return Ok::<_, color_eyre::eyre::Report>((jar, Redirect::to("/")));
        };
        let bearer = state
            .oauth2
            .exchange_code(AuthorizationCode::new(code.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(
                jar.get("pkce")
                    .map(|c| c.value().to_string())
                    .wrap_err("no pkce")?,
            ))
            .request_async(&state.http_client)
            .await?;
        let rtok = bearer.access_token();
        let userinfo: UserInfoClaims<
            openidconnect::EmptyAdditionalClaims,
            openidconnect::core::CoreGenderClaim,
        > = state
            .oauth2
            .user_info(rtok.clone(), None)?
            .request_async(&state.http_client)
            .await?;
        let user_tok =
            name_to_user_token(&state.db, userinfo.email().wrap_err("no email")?.as_ref()).await?;

        let mut cookie = Cookie::new(AUTH_COOKIE, user_tok);
        cookie.set_secure(false);
        cookie.set_path("/");

        eyre::Result::Ok((jar.add(cookie), Redirect::to("/")))
    };
    match inner().await {
        Ok(ret) => Ok(ret),
        Err(e) => {
            error!("Oauth2 Callback Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn logout(
    State(state): State<AppState>,
    mut cookie: axum_extra::extract::CookieJar,
) -> (axum_extra::extract::CookieJar, &'static str) {
    let v: Vec<String> = cookie.iter().map(|c| c.name().to_string()).collect();
    for s in v {
        cookie = cookie.remove(s);
    }
    (cookie, "Logged out")
}

pub async fn name_to_user_token(
    db: &crate::database::Database,
    name: &str,
) -> color_eyre::Result<String> {
    let user = db.get_user_from_name(name).await?;
    if let Some(user) = user {
        return Ok(user.token);
    }
    db.create_user(name).await.map(|s| s.1)
}
