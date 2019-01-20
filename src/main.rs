//!
//! The application binary.
//!

#![feature(generators, generator_trait)]

mod broker;
mod error;
mod http;
mod state;

use std::{env, io, num};

use actix_web::middleware::Logger;
use log::*;

#[derive(Debug)]
enum Error {
    MissingEnvironmentVariable(&'static str, env::VarError),
    InvalidEnvironmentVariable(&'static str, num::ParseIntError),
    ActixHttp(io::Error),
    ActixSystem(i32),
}

///
/// RUST_LOG='carrier=info,actix_web=info,conduit2=info'
///
fn main() -> Result<(), Error> {
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .init();

    let port = env::var("HTTP_PORT")
        .map_err(|error| Error::MissingEnvironmentVariable("HTTP_PORT", error))?;
    let port: u16 = port
        .parse()
        .map_err(|error| Error::InvalidEnvironmentVariable("HTTP_PORT", error))?;

    let system = actix::System::new(env!("CARGO_PKG_NAME"));
    let poll = osaka::Poll::new();

    let broker = actix::Arbiter::start(move |_| broker::BrokerActor::new(poll));
    let state = crate::state::State::new(broker);

    let address = format!("{}:{}", "0.0.0.0", port);
    let log_format = "%a %r => %s (%bB sent)";
    actix_web::server::new(move || {
        actix_web::App::with_state(state.clone())
            .middleware(Logger::new(log_format))
            .configure(http::router)
    })
    .bind(&address)
    .map(move |server| {
        info!("HTTP server has been started at {}", address);
        server
    })
    .map_err(Error::ActixHttp)?
    .start();

    let code = system.run();
    if code == 0 {
        Ok(())
    } else {
        Err(Error::ActixSystem(code))
    }
}
