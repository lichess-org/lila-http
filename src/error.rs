use axum::{body, http::StatusCode, response::Response};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No such tournament")]
    NotFoundError,
    #[error("lila fetch failed with {0}")]
    ReqwestError(Arc<reqwest::Error>),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(body::boxed(body::Full::from(self.to_string())))
            .unwrap()
    }
}
