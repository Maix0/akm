use axum::http::StatusCode;

pub mod client;
pub mod key;
pub mod utils;

pub trait ErrorToStatusCode<T> {
    fn to_status(self) -> std::result::Result<T, StatusCode>;
}

impl<T, E: std::fmt::Display> ErrorToStatusCode<T> for Result<T, E> {
    #[track_caller]
    fn to_status(self) -> std::result::Result<T, StatusCode> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                tracing::error!("Error: {e}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
