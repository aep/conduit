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
    #[fail(display = "Internal error: {}", _0)]
    InternalErrorActix(actix::MailboxError),
    #[fail(display = "Internal error: {}", _0)]
    InternalErrorCarrier(carrier::error::Error),
}

impl From<actix::MailboxError> for Error {
    fn from(error: actix::MailboxError) -> Self {
        Error::InternalErrorActix(error)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::MissingParameter(..) => HttpResponse::BadRequest().json(JsonError::new(self)),
            Error::IdentityParsingError(..) => {
                HttpResponse::BadRequest().json(JsonError::new(self))
            }
            Error::InternalErrorActix(..) => {
                HttpResponse::InternalServerError().json(JsonError::new(self))
            }
            Error::InternalErrorCarrier(..) => {
                HttpResponse::InternalServerError().json(JsonError::new(self))
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
