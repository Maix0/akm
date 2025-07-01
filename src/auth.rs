use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use log::error;

use crate::{
    database::users::{TableUsers, UserId},
    state::AppState,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct UserAuth(crate::database::users::UserId);

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
            Ok(None) => Err((cookies.remove(AUTH_COOKIE), StatusCode::FORBIDDEN)),
            Ok(Some(v)) => Ok(Self(v)),
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
