//!
//! The broker actor.
//!

use actix::{Actor, Context, Handler, Message};
use log::info;
use osaka::{osaka, Future, FutureResult, Poll};

use carrier::{endpoint::Endpoint, error::Error, identity::Identity};

pub struct BrokerActor {
    endpoint: Option<Endpoint>,
}

impl BrokerActor {
    pub fn new(poll: Poll) -> Self {
        match Self::endpoint(poll).run() {
            Ok(endpoint) => {
                info!("Carrier Endpoint Actor has been started");
                Self {
                    endpoint: Some(endpoint),
                }
            },
            Err(error) => panic!(error),
        }
    }

    #[osaka]
    fn endpoint(poll: Poll) -> Result<Endpoint, Error> {
        let config = carrier::config::load()?;

        let mut endpoint = carrier::endpoint::EndpointBuilder::new(&config)?.connect(poll.clone());
        let endpoint: Endpoint = osaka::sync!(endpoint)?;

        Ok(endpoint)
    }
}

pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

pub struct BrokerActorMessage(pub Identity);

impl Message for BrokerActorMessage {
    type Result = Result<DeviceStatus, Error>;
}

impl Actor for BrokerActor {
    type Context = Context<Self>;
}

impl Handler<BrokerActorMessage> for BrokerActor {
    type Result = Result<DeviceStatus, Error>;

    fn handle(&mut self, msg: BrokerActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let mut endpoint = self.endpoint.take().expect("Endpoint acquisition bug");
        endpoint.connect(msg.0)?;
        let (endpoint, result) = outgoing_connect(endpoint).run();
        self.endpoint = Some(endpoint);
        result
    }
}

#[osaka]
pub fn outgoing_connect(mut endpoint: Endpoint) -> (Endpoint, Result<DeviceStatus, Error>) {
    loop {
        match Endpoint::poll(&mut endpoint) {
            FutureResult::Done(Ok(carrier::endpoint::Event::OutgoingConnect(response))) => {
                info!(
                    "OutgoingConnect debug: {:?} {} {}",
                    response.identity,
                    response.cr.is_some(),
                    response.requester.is_some()
                );
                let status = match response.cr {
                    Some(..) => DeviceStatus::Online,
                    None => DeviceStatus::Offline,
                };
                return (endpoint, Ok(status));
            }
            FutureResult::Done(Ok(carrier::endpoint::Event::Disconnect { .. })) => {
                info!("Disconnect: undefined behaviour");
                return (endpoint, Ok(DeviceStatus::Unknown));
            }
            FutureResult::Done(Ok(carrier::endpoint::Event::IncommingConnect(_))) => {
                info!("IncomingConnect: ignored");
            }
            FutureResult::Done(Err(e)) => {
                return (endpoint, Err(e));
            }
            FutureResult::Again(y) => {
                yield y;
            }
        }
    }
}
