use google_cloud::pubsub;
use serde::Serialize;
use warp::Filter;

use chrono::Local;

#[derive(Clone, Serialize, Debug)]
struct State {
    value: String,
    timestamp: String,
}

#[derive(Clone, Serialize, Debug)]
struct Response {
    current: State,
    history: Vec<State>,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let mut client = match pubsub::Client::new("wearebrews").await {
        Ok(c) => c,
        Err(e) => panic!("Unable to create pubsub client: {:?}", e),
    };

    let sub = match client.subscription("anyonethere").await {
        Ok(s) => s,
        Err(e) => panic!("Unable to load subscription: {:?}", e),
    };

    let mut sub: pubsub::Subscription = match sub {
        Some(s) => s,
        None => panic!("No subscription found"),
    };
    let (tx, rx) = tokio::sync::watch::channel::<Response>(Response {
        current: State {
            value: "UNKNOWN".to_string(),
            timestamp: "".to_string(),
        },
        history: vec![],
    });
    let tx: tokio::sync::watch::Sender<Response> = tx;
    let rx: tokio::sync::watch::Receiver<Response> = rx;

    tokio::task::spawn(async move {
        let cors = warp::cors()
            .allow_origin("https://haavardm.github.io")
            .allow_origin("http://localhost:8080")
            .allow_methods(vec!["GET"]);
        let state_handler = warp::path::end()
            .map(move || warp::reply::json(&*rx.borrow()))
            .with(cors);
        println!("Starting service");
        warp::serve(state_handler).run(([0, 0, 0, 0], 3030)).await;
    });

    tokio::task::spawn(async move {
        let mut resp = Response {
            current: State {
                value: "".to_string(),
                timestamp: "".to_string(),
            },
            history: Vec::new(),
        };
        resp.history.reserve(20);
        loop {
            let mut received = match sub.receive().await {
                Some(m) => m,
                None => panic!("Unable to receive message"),
            };
            let date = Local::now().format("%Y-%m-%d %H:%M:%S");
            let attr: &std::collections::HashMap<String, String> = received.attributes();
            if Some(&"pir".to_string()) == attr.get("subFolder") {
                let msg = match String::from_utf8(received.data().to_owned()) {
                    Ok(s) => State {
                        value: s,
                        timestamp: Local::now()
                            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    },
                    Err(e) => panic!("unable to decode message: {:?}", e),
                };
                if resp.history.len() >= 20 {
                    resp.history.remove(0);
                }
                resp.history.push(msg.clone());
                resp.current = msg;
                tx.broadcast(resp.clone()).expect("Unable to send message");
                println!(
                    "Accepting message {} at {}",
                    std::str::from_utf8(received.data()).expect("unable to decode utf8 string"),
                    date
                );
            } else {
                println!(
                    "Dropping message from subFolder {} at {}",
                    attr.get("subFolder").unwrap_or(&String::from("NONE")),
                    date
                )
            }
            received.ack().await.expect("Unable to ack message");
        }
    })
    .await
    .expect("Unable to spawn pubsub worker");
}
