//!
//! The HTTP server state.
//!

use actix::Addr;

use crate::broker::BrokerActor;

#[derive(Clone)]
pub struct State {
    pub broker: Addr<BrokerActor>,
}

impl State {
    pub fn new(broker: Addr<BrokerActor>) -> Self {
        Self { broker }
    }
}
