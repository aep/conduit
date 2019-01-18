//!
//! The HTTP server state.
//!

use std::sync::{Arc, Mutex};

use carrier::{error::Error, identity::Identity};

use crate::broker::Broker;

#[derive(Clone)]
pub struct State {
    broker: Arc<Mutex<Broker>>,
}

impl State {
    pub fn new(broker: Broker) -> Self {
        Self {
            broker: Arc::new(Mutex::new(broker)),
        }
    }

    pub fn broker_connect(&self, identity: Identity) -> Result<(), Error> {
        self.broker
            .clone()
            .lock()
            .expect("Broker mutex bug")
            .connect(identity)
    }
}
