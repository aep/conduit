//#[osaka]
//fn handler(poll: osaka::Poll, mut stream: carrier::endpoint::Stream) {
//    use prost::Message;
//
//    let m = osaka::sync!(stream);
//    let headers = carrier::headers::Headers::decode(&m).unwrap();
//    info!("pubres: {:?}", headers);
//
//    let sc = carrier::proto::SubscribeChange::decode(osaka::sync!(stream)).unwrap();
//
//    match sc.m {
//        Some(carrier::proto::subscribe_change::M::Publish(carrier::proto::Publish {
//            identity,
//            xaddr,
//        })) => {}
//        Some(carrier::proto::subscribe_change::M::Unpublish(carrier::proto::Unpublish {
//            identity,
//        })) => {}
//        Some(carrier::proto::subscribe_change::M::Supersede(_)) => {
//            panic!("subscriber superseded");
//        }
//        None => (),
//    }
//}

//    let shadow: carrier::identity::Address = "oSkTq69F7jzKshUPzMpb8Qj45RBnCxqTp4R16LkEEpYHe6T"
//        .parse()
//        .unwrap();

//    let broker = ep.broker();
//    ep.open(
//        broker,
//        carrier::headers::Headers::with_path("/carrier.broker.v1/broker/subscribe"),
//        |poll, mut stream| {
//            stream.message(carrier::proto::SubscribeRequest {
//                shadow: shadow.as_bytes().to_vec(),
//                filter: Vec::new(),
//            });
//            handler(poll, stream)
//        },
//    );

//    loop {
//        match osaka::sync!(ep)? {
//            carrier::endpoint::Event::Disconnect { .. } => (),
//            carrier::endpoint::Event::OutgoingConnect(_) => (),
//            carrier::endpoint::Event::IncommingConnect(q) => {
//                info!("ignoring incomming connect {}", q.identity);
//            }
//        };
//    }
