#![feature(generators, generator_trait)]

extern crate carrier;
extern crate env_logger;
extern crate osaka;

extern crate log;
extern crate prost;
extern crate redis;
extern crate redis_cluster_rs;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate hwaddr;
extern crate actix_web;
extern crate mio_extras;

use carrier::{error::Error, identity, util::defer, conduit::Conduit, config};
use log::{info, warn};
use osaka::{osaka, FutureResult};
use prost::Message;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use redis::Commands;
use std::sync::Arc;
use serde_derive::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::{http, server, App, Path, Responder, State};
use std::thread;
use mio_extras::channel;
use osaka::Future;

#[derive(Deserialize, Serialize, Debug)]
pub struct OdhcpHost {
    pub m : String,
    pub ip: String,
    pub l:  String,
    pub n:  String,
}

#[derive(Deserialize, Debug)]
pub struct Odhcp{
    private: Vec<OdhcpHost>,
    public:  Vec<OdhcpHost>,
}

mod uitypes;

pub struct HttpApiCall {
    pub identity:   identity::Identity,
    pub headers:    carrier::headers::Headers,
}

#[derive(Clone)]
struct MyState {
    tx: channel::Sender<HttpApiCall>,
}

fn via(state: State<MyState>, p: Path<(String, String)>) -> impl Responder {
    if let Ok(identity) = p.0.parse() {
        state.tx.send(HttpApiCall{
            identity,
            headers: carrier::headers::Headers::with_path("/v1/assemble"),
        }).unwrap();
        String::from("{\"ok\": true}")
    } else {
        String::from("{\"ok\": false}")
    }
}



#[osaka]
pub fn main() -> Result<(), Error> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let (tx, rx) = channel::channel();
    let state = MyState{tx};
    thread::spawn(move ||{
        server::new(
            move || App::with_state(state.clone())
            .route("/via/{identity}/{tail:.*}", http::Method::POST, via))
            .bind("0.0.0.0:8052").unwrap()
            .run();
    });

    let redis_url = env::var("REDIS").unwrap_or("redis://127.0.0.1/".to_string());
    let redis_client = redis_cluster_rs::Client::open(vec![redis_url.as_str()]).unwrap();
    let redis_  = Arc::new(redis_client.get_connection().expect("redis"));


    let config = config::load().unwrap();
    let poll = osaka::Poll::new();
    let mut conduit = Conduit::new(poll.clone(), config);
    let mut conduit = osaka::sync!(conduit)?;

    let redis = redis_.clone();
    conduit.schedule_raw(
        Duration::from_secs(10),
        carrier::headers::Headers::with_path("/v1/unixbus"),
        move |identity: &carrier::identity::Identity, msg: Vec<u8>| {
            let msg = String::from_utf8_lossy(&msg);
            let msg : Vec<&str> = msg.split_terminator("\n").collect();
            if msg.len() > 1 && msg[0] == "/odhcpd"{
                let msg = msg[1..].join("\n");
                match serde_json::from_str::<Odhcp>(&msg) {
                    Ok(v) => {
                        println!("{:?}", v);
                        for v in v.public{
                            let key = format!("{}:dhcp:{}", identity, v.m);
                            let l = v.l.parse().unwrap_or(60000);
                            let _ : () = redis.hset_multiple(&key, &[
                                ("mac",     v.m),
                                ("ip",      v.ip),
                                ("lease",   v.l),
                                ("name",    v.n),
                                ("zone",    "private".to_string()),
                            ]).unwrap();
                            let _ : () = redis.expire(&key, l).unwrap();
                        }
                        for v in v.private {
                            let key = format!("{}:dhcp:{}", identity, v.m);
                            let l = v.l.parse().unwrap_or(60000);
                            let _ : () = redis.hset_multiple(&key, &[
                                ("mac",     v.m),
                                ("ip",      v.ip),
                                ("lease",   v.l),
                                ("name",    v.n),
                                ("zone",    "private".to_string()),
                            ]).unwrap();
                            let _ : () = redis.expire(&key, l).unwrap();
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }

            println!("{} => {:#?}", identity, msg);
        }
    );


    let redis = redis_.clone();
    conduit.schedule(
        Duration::from_secs(10),
        carrier::headers::Headers::with_path("/v1/netsurvey"),
        move |identity: &carrier::identity::Identity, msg: carrier::proto::NetSurvey| {

            let mut stations = HashMap::new();

            for i  in &msg.wifi {
                for st in &i.stations {

                    let hwa = st.address.replace(":", "");

                    let key = format!("{}:dhcp:{}", identity, hwa);
                    let ip = redis.hget(&key, "ip").ok();
                    let host_name = redis.hget(&key, "name").ok();


                    let hwa : Vec<u8> = hwa.as_bytes().chunks(2).map(|b|u8::from_str_radix(&String::from_utf8_lossy(b),16).unwrap_or(0)).collect();
                    let hwa = hwaddr::HwAddr::from(&hwa[..]);


                    let station = uitypes::Station {
                        mac:    st.address.clone(),
                        vendor: hwa.producer().map(|p|p.name.to_string()),
                        ip,
                        host_name,
                        signal: st.signal.get(0).cloned(),
                        signal_avg: st.signal_avg.get(0).cloned(),
                        radio: if i.name.contains("-g") {
                            Some("g".to_string())
                        } else if i.name.contains("-a") {
                            Some("a".to_string())
                        } else {
                            None
                        }
                    };

                    let zone : Vec<&str> = i.name.split("-").collect();;
                    let zone = if zone.len() == 3 {
                        zone[1].to_string()
                    } else {
                        "other".to_string()
                    };

                    stations.entry(zone).or_insert(Vec::new()).push(station);


                }
            }

            let _ : () = redis.hset(format!("{}", identity), "stations", serde_json::to_string(&stations).unwrap()).unwrap();

            let t = format!(r#"{{"timestamp":{}, "state": "up", "carrier": true}}"#,
                            SystemTime::now().duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs()
                            );
            let _ : () = redis.hset(format!("{}", identity), "up", &t).unwrap();
            println!("{} => {:#?}", identity, msg);
        }
    );


    /*
    let redis = redis_.clone();
    conduit.schedule(
        Duration::from_secs(10),
        carrier::headers::Headers::with_path("/v1/sysinfo"),
        move |identity: &carrier::identity::Identity, msg: carrier::proto::Sysinfo| {
            println!("{} => {:#?}", identity, msg);

            if let Some(sw0) =  msg.switch.get(0) {
                let mut switch  = HashMap::new();

                for port in &sw0.ports {
                    switch.insert(format!("{}", port.port), uitypes::SwitchPort{
                        speed:      port.speed.clone(),
                        link:       {if port.link { "up" } else {"down"}}.into(),
                        gateway:    false,
                    });
                }

                let mut iproute = HashMap::new();

                let net = uitypes::Net{
                    switch,
                    iproute,
                };

                let _ : () = redis.hset(format!("{}", identity), "net", serde_json::to_string(&net).unwrap()).unwrap();
            }
        }
    );
    */

    let t1 = poll
        .register(&rx, osaka::mio::Ready::readable(), osaka::mio::PollOpt::level())
        .unwrap();
    let yy = poll.again(t1, None);



    loop {
        let mut y = match conduit.poll() {
            FutureResult::Done(r) => return r,
            FutureResult::Again(y) => y,
        };

        match rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            Ok(v) => {
                println!("calling {:?} => {:?}", v.identity, v.headers);
                conduit.call(v.identity, v.headers, move |identity: &carrier::identity::Identity, msg: carrier::proto::Empty| {

                });

            }
        }


        y.merge(yy.clone());
        yield y;
    }

    Ok(())
}







