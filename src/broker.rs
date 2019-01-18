//!
//! The broker wrapper.
//!

use osaka::osaka;

use carrier::{endpoint::Endpoint, error::Error, identity::Identity};

pub struct Broker {
    endpoint: Endpoint,
}

unsafe impl Send for Broker {}

impl Broker {
    pub fn create() -> Result<Self, Error> {
        let poll = osaka::Poll::new();
        let endpoint = endpoint(poll).run()?;
        Ok(Self { endpoint })
    }

    pub fn connect(&mut self, identity: Identity) -> Result<(), Error> {
        self.endpoint.connect(identity)
    }
}

#[osaka]
fn endpoint(poll: osaka::Poll) -> Result<Endpoint, Error> {
    let config = carrier::config::load()?;

    let mut endpoint = carrier::endpoint::EndpointBuilder::new(&config)?.connect(poll.clone());
    let endpoint: Endpoint = osaka::sync!(endpoint)?;

    Ok(endpoint)
}
