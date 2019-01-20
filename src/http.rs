//!
//! The HTTP router and handlers.
//!

use actix_web::{App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse};
use futures::future::Future;
use serde_derive::Serialize;

use crate::{broker::{BrokerActorMessage, DeviceStatus}, error::Error, state::State};

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

fn status(request: HttpRequest<State>) -> Result<FutureResponse<HttpResponse, Error>, Error> {
    let state: &State = request.state();

    let identity = request
        .query()
        .get("identity")
        .ok_or(Error::MissingParameter("identity"))?
        .to_owned();
    let identity: carrier::identity::Identity = identity
        .parse()
        .map_err(|error| Error::IdentityParsingError(identity, error))?;

    Ok(state
        .broker
        .send(BrokerActorMessage(identity))
        .from_err()
        .and_then(|result| {
            result
                .map_err(Error::InternalErrorCarrier)
                .and_then(|data| {
                    let result = match data {
                        DeviceStatus::Online => StatusResponse::new("Online".to_string()),
                        DeviceStatus::Offline => StatusResponse::new("Offline".to_string()),
                        DeviceStatus::Unknown => StatusResponse::new("Unknown".to_string()),
                    };
                    Ok(HttpResponse::Ok().json(result))
                })
        })
        .responder())
}
