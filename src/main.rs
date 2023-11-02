use hyper::body::Bytes;
use hyper::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::io;
use std::io::{stdin, Write};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use warp::Filter;

#[derive(Clone)]
struct Senders {
    pub cli: Arc<Sender<String>>,
    pub timer: Arc<Sender<String>>,
    pub server: Arc<Sender<String>>,
}

unsafe impl Send for Senders {}

fn with_senders(
    sender: Senders,
) -> impl Filter<Extract = (Senders,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

fn handle_twitch_event(
    headers: HeaderMap,
    body: serde_json::Value,
    senders: Senders,
) -> Result<String, String> {
    let body = if let Some(b) = body.as_object() {
        b
    } else {
        return Err("".to_string());
    };
    println!("\n\n{:?}", body.keys().collect::<Vec<_>>());
    let subscription = body.get("subscription").unwrap().as_object().unwrap();
    println!("\n\n{:?}", subscription.keys().collect::<Vec<_>>());
    for (k, v) in subscription {
        println!("{}, {:?}", k, v);
    }
    Ok("".to_string())
}

fn handle_kofi_event() {}

async fn handle_url_encoded(
    headers: HeaderMap,
    body: HashMap<String, String>,
    senders: Senders,
) -> Result<String, String> {
    let data = body.get("data").ok_or("nani=".to_string())?;
    let data: Value = serde_json::from_str(&data).or(Err("NANI???".to_string()))?;
    let data = data.as_object().ok_or("NANI????".to_string())?;
    for (k, v) in data {
        println!("{}, {:?}", k, v);
    }
    Ok("".to_string())
}

async fn handle_json(
    headers: HeaderMap,
    body: serde_json::Value,
    senders: Senders,
) -> Result<String, String> {
    handle_twitch_event(headers, body, senders)
}

async fn server(ip: [u8; 4], port: u16, senders: Senders) {
    println!("Starting server...");
    let routes = warp::any()
        .and(warp::header::headers_cloned())
        .and(warp::body::json())
        .and(with_senders(senders.clone()))
        .and_then(move |headers, body, senders| async move {
            Ok::<_, Infallible>(handle_json(headers, body, senders).await)
        })
        .or(warp::any()
            .and(warp::header::headers_cloned())
            .and(warp::body::form())
            .and(with_senders(senders.clone()))
            .and_then(move |headers, body, senders| async move {
                Ok::<_, Infallible>(handle_url_encoded(headers, body, senders).await)
            }));
    let _ = senders.cli.send("server".to_string());
    println!("{:?}", warp::serve(routes).run((ip, port)).await);
}

async fn timer(settings: bool, senders: Senders) {
    println!("Starting timer...");
    let _ = senders.cli.send("timer".to_string());
}

#[tokio::main]
async fn main() {
    let (server_tx, server_rx) = mpsc::channel();
    let (cli_tx, cli_rx) = mpsc::channel();
    let (timer_tx, timer_rx) = mpsc::channel();
    let senders_server = Senders {
        cli: Arc::new(cli_tx.clone()),
        timer: Arc::new(timer_tx.clone()),
        server: Arc::new(server_tx.clone()),
    };
    let senders_timer = senders_server.clone();
    let server = tokio::spawn(async move {
        server([127, 0, 0, 1], 8080, senders_server).await;
    });
    let timer = tokio::spawn(async move {
        timer(false, senders_timer).await;
    });

    let rx = cli_rx;
    let _ = rx.recv();
    let _ = rx.recv();

    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = stdin().read_line(&mut input);
        println!("{}", input);
        let input = input.trim().to_ascii_lowercase();
        let split = input.split(" ").collect::<Vec<_>>();
        if split.len() < 1 {
            continue;
        }
        match split[0] {
            "stop" => {
                break;
            }
            "pause" => {
                println!("Pausing timer ...");
                let _ = timer_tx.send("pause".to_string());
            }
            "addtime" | "add" => {
                if split.len() < 2 {
                    continue;
                } else {
                    println!("Adding {} minutes", split[1]);
                }
            }
            _ => {
                println!("unrecognized command \"{}\"", input);
            }
        }
    }

    server.await.expect("");
    timer.await.expect("");
}
