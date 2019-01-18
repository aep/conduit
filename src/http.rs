//!
//! The HTTP router.
//!

use actix_web::{App, HttpRequest, HttpResponse};
use serde_derive::Serialize;

use crate::{error::Error, state::State};

pub fn router(application: App<State>) -> App<State> {
    application.scope("/api/v1", |scope| {
        scope.resource("/status", |resource| resource.get().with(status))
    })
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub response: String,
}

impl StatusResponse {
    pub fn new(response: String) -> Self {
        Self { response }
    }
}

fn status(request: HttpRequest<State>) -> Result<HttpResponse, Error> {
    let state: &State = request.state();

    let identity = request
        .query()
        .get("identity")
        .ok_or(Error::MissingParameter("identity"))?
        .to_owned();
    let identity: carrier::identity::Identity = identity
        .parse()
        .map_err(|error| Error::IdentityParsingError(identity, error))?;

    let result = match state.broker_connect(identity) {
        Ok(..) => StatusResponse::new(format!("Online")),
        Err(..) => StatusResponse::new(format!("Offline")),
    };

    Ok(HttpResponse::Ok().json(result))
}
