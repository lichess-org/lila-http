use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum HttpResponseError {
    #[error("{0}")]
    HTTP(StatusCode),
}

impl IntoResponse for HttpResponseError {
    fn into_response(self) -> Response {
        match self {
            HttpResponseError::HTTP(sc) => (sc, sc.to_string()).into_response(),
        }
    }
}

pub fn not_found() -> HttpResponseError {
    HttpResponseError::HTTP(StatusCode::NOT_FOUND)
}
