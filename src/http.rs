use axum::{
    body,
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
            HttpResponseError::HTTP(sc) => Response::builder()
                .status(sc)
                .body(body::boxed(body::Full::from(sc.to_string())))
                .unwrap(),
        }
    }
}

pub fn not_found() -> HttpResponseError {
    HttpResponseError::HTTP(StatusCode::NOT_FOUND)
}
