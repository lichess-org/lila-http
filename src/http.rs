use serde::Serialize;
use axum::{
    body, http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::{Error as ThisError};
use serde_json::{Error as SerdeJsonError};

#[derive(ThisError, Debug)]
pub enum HttpResponseError {
    #[error("{0}")]
    HTTP(StatusCode),
    #[error("[SERDE] JSON Error: {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
}

impl IntoResponse for HttpResponseError {
    fn into_response(self) -> Response {
        match self {
            HttpResponseError::SerdeJsonError(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(body::boxed(body::Full::from(err.to_string())))
                .unwrap(),
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

pub struct Json(String);

impl Json {
    pub fn from<'a, V>(value: &'a V) -> Result<Json, HttpResponseError>
        where V: Serialize {
        Ok(Json(serde_json::to_string(&value)?))
    }
}

impl IntoResponse for Json {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(body::boxed(body::Full::from(self.0)))
            .unwrap()
    }
}

