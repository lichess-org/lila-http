use axum::{body, http::StatusCode, response::Response};
use std::{error::Error as StdError, fmt};
#[derive(Debug)]

pub enum Error {
    NotFoundError,
    LilaError(reqwest::Error),
}
