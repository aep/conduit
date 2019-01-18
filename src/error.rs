//!
//! HTTP errors.
//!

use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
use serde_derive::Serialize;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Missing parameter '{}'", _0)]
    MissingParameter(&'static str),
    #[fail(display = "Identity {} parsing error: {}", _0, _1)]
    IdentityParsingError(String, carrier::error::Error),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::MissingParameter(..) => HttpResponse::BadRequest().json(JsonError::new(self)),
            Error::IdentityParsingError(..) => {
                HttpResponse::BadRequest().json(JsonError::new(self))
            }
        }
    }
}

#[derive(Serialize)]
struct JsonError {
    error: String,
}

impl JsonError {
    pub fn new(error: &Error) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}
