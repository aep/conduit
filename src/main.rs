#![feature(generators, generator_trait)]

extern crate carrier;
extern crate osaka;
extern crate tinylogger;
extern crate log;
extern crate prost;

use carrier::error::Error;
use log::{
    info
};
use osaka::{
    osaka,
};
use std::env;


pub fn main() -> Result<(), Error> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    tinylogger::init().ok();

    let poll  = osaka::Poll::new();
    main_async(poll).run()
}


#[osaka]
fn handler(poll: osaka::Poll, mut stream: carrier::endpoint::Stream) {
    use prost::Message;

    let m = osaka::sync!(stream);
    let headers = carrier::headers::Headers::decode(&m).unwrap();
    info!("pubres: {:?}", headers);

    let sc = carrier::proto::SubscribeChange::decode(osaka::sync!(stream)).unwrap();

    match sc.m {
        Some(carrier::proto::subscribe_change::M::Publish(carrier::proto::Publish{identity, xaddr})) => {
        },
        Some(carrier::proto::subscribe_change::M::Unpublish(carrier::proto::Unpublish{identity})) => {
        }
        Some(carrier::proto::subscribe_change::M::Supersede(_)) => {
            panic!("subscriber superseded");
        },
        None =>(),
    }

}

#[osaka]
pub fn main_async(poll: osaka::Poll) -> Result<(), Error> {

    let shadow : carrier::identity::Address = "oT5kQdir3RqedEtVkvqcJA13Ze8czCoobBoCMjSj7MDPuSj".parse().unwrap();

    let config  = carrier::config::load()?;

    let mut ep = carrier::endpoint::EndpointBuilder::new(&config)?.connect(poll.clone());
    let mut ep = osaka::sync!(ep)?;

    let broker = ep.broker();
    ep.open(
        broker,
        carrier::headers::Headers::with_path("/carrier.broker.v1/broker/subscribe"),
        |poll, mut stream| {
            stream.message(carrier::proto::SubscribeRequest {
                shadow: shadow.as_bytes().to_vec(),
                filter: Vec::new(),
            });
            handler(poll, stream)
        },
        );

    loop {
        match osaka::sync!(ep)? {
            carrier::endpoint::Event::Disconnect{..} => (),
            carrier::endpoint::Event::OutgoingConnect(_) => (),
            carrier::endpoint::Event::IncommingConnect(q) => {
                info!("ignoring incomming connect {}", q.identity);
            }
        };
    }

    Ok(())
}
