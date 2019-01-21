#![feature(generators, generator_trait)]

extern crate carrier;
extern crate osaka;
extern crate tinylogger;
extern crate log;
extern crate prost;
extern crate actix_web;
extern crate mio_extras;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;


use mio_extras::channel;
use actix_web::{http, server, App, Path, Responder, State};
use std::thread;
use carrier::{
    error::Error,
};
use log::{
    info
};
use osaka::{
    mio,
    osaka,
    Future,
    FutureResult,
};
use std::env;
use std::sync::mpsc;
use std::collections::HashMap;



#[derive(Serialize)]
pub struct ApiResponse {
    pub identity: String,
    pub connectok: bool,
}

#[derive(Clone)]
struct MyState {
    tx: channel::Sender<(carrier::Identity, mpsc::Sender<ApiResponse>)>,
}


fn index(state: State<MyState>, info: Path<String>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    let r = info.parse()
        .map_err(|e|format!("{}",e))
        .and_then(|identity : carrier::Identity|{
            state.tx
                .send((identity, tx))
                .map_err(|e|format!("{}",e))
                .and_then(|_|{
                    rx.recv()
                        .map_err(|e|format!("{}",e))
                        .and_then(|r|{
                            serde_json::to_string(&r)
                                .map_err(|e|format!("{}",e))
                        })
                })

        });

    match r {
        Ok(v) => v,
        Err(v) => format!("{{\"error\": \"{}\" }}", v),
    }

}

pub fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    tinylogger::init().ok();


    let (tx, rx) = channel::channel();

    thread::spawn(move || {
        let poll  = osaka::Poll::new();
        main_async(poll, rx).run().unwrap();
    });

    let state = MyState{tx};
    server::new(
        move || App::with_state(state.clone())
            .route("/{id}/info.json", http::Method::GET, index))
        .bind("0.0.0.0:8084").unwrap()
        .run();
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
pub fn main_async(poll: osaka::Poll, info_req: channel::Receiver<(carrier::Identity, mpsc::Sender<ApiResponse>)>) -> Result<(), Error> {


    let mut open_connects = HashMap::new();

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


    let t1 = poll
        .register(&info_req , mio::Ready::readable(), mio::PollOpt::level())
        .unwrap();
    let yy = poll.again(t1, None);


    loop {
        match info_req.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            Ok(v) => {
                info!("got req for {}", v.0);
                ep.connect(v.0.clone())?;
                open_connects.insert(v.0, (v.1, ));
            }
        };

        match ep.poll() {
            FutureResult::Done(Ok(carrier::endpoint::Event::Disconnect{..})) => (),
            FutureResult::Done(Ok(carrier::endpoint::Event::OutgoingConnect(q))) => {
                if let Some(v) = open_connects.remove(&q.identity) {
                    info!("connect res for open connect");
                    v.0.send(ApiResponse{
                        identity: format!("{}", q.identity),
                        connectok: match q.cr {
                            None => false,
                            Some(cr) => cr.ok,
                        },
                    }).unwrap();
                }
            },
            FutureResult::Done(Ok(carrier::endpoint::Event::IncommingConnect(q))) => {
                info!("ignoring incomming connect {}", q.identity);
            },
            FutureResult::Done(Err(e)) => return Err(e),
            FutureResult::Again(mut y) => {
                y.merge(yy.clone());
                yield y;
            }
        };
    }

    Ok(())
}
