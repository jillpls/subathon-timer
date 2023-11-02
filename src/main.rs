use warp::Filter;
use std::convert::Infallible;
use std::io::{stdin, Write};
use std::net::SocketAddr;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use hyper::{Body, Request, Response, Server};
use hyper::body::{Bytes, HttpBody};
use hyper::service::{make_service_fn, service_fn};
use urlencoding::decode;
use futures::executor::block_on;
use tokio::sync::Mutex;

#[derive(Clone)]
struct Senders {
    pub cli_sender : Arc<Sender<String>>,
    pub timer_sender : Arc<Sender<String>>,
    pub server_sender : Arc<Sender<String>>,
}

unsafe impl Send for Senders {}

fn with_senders(
    sender: Senders,
) -> impl Filter<Extract = (Senders,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || sender.clone())
}

async fn handle(req: Bytes, senders : Senders) -> Result<String, String> {
    Ok("".to_string())
}

async fn server(ip: [u8; 4], port: u16, senders: Senders) {
    println!("aaaa");
    let routes = warp::any()
        .and(warp::body::bytes())
        .and(with_senders(senders.clone()))
        .and_then( move | body, senders| async move {
        Ok::<_, Infallible>(handle(body, senders).await)
    });
    warp::serve(routes).run((ip, port)).await;
    //
    //
    //
    // println!("aaaa");
    // let addr = SocketAddr::from((ip, port));
    // let make_service = make_service_fn(move |_conn| async move {
    //     Ok::<_, Infallible>(service_fn(move |req| handle(req, senders.clone())))
    // });
    // let server = Server::bind(&addr).serve(make_service);
    // if let Err(e) = server.await {
    //     eprintln!("server error: {}", e);
    // }
}

async fn timer(settings : bool, server_tx: Sender<String>, cli_tx : Sender<String>, rx: Receiver<String>) {
    let _ = cli_tx.send("".to_string());
    println!("aaaa");
}

#[tokio::main]
async fn main() {
    let (server_tx, server_rx) = mpsc::channel();
    let (cli_tx, cli_rx) = mpsc::channel();
    let (timer_tx, timer_rx) = mpsc::channel();
    let connections_server = Senders {
        cli_sender : Arc::new(cli_tx.clone()),
        timer_sender : Arc::new(timer_tx.clone()),
        server_sender : Arc::new(server_tx.clone()),
    };
    let server_to_timer_tx = timer_tx.clone();
    let server_to_cli_tx = cli_tx.clone();
    let timer_to_server_tx = server_tx.clone();
    let timer_to_cli_tx = cli_tx.clone();
    let server = tokio::spawn(async move {
        block_on(server([127, 0, 0, 1], 8080, connections_server))
    });
    let timer = tokio::spawn(async move {
        block_on(timer(false, timer_to_server_tx, timer_to_cli_tx, timer_rx))
    });

    let rx = cli_rx;
    let r1 = rx.recv();
    let r2 = rx.recv();

    while true {
        print!("> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        stdin().read_line(&mut input);
        println!("{}", input);
        let input = input.trim().to_ascii_lowercase();
        let split = input.split(" ").collect::<Vec<_>>();
        if split.len() < 1 { continue; }
        match split[0] {
            "pause" => {
                println!("Pausing timer ...");
                let _ = timer_tx.send("pause".to_string());
            },
            "addtime" | "add" => {
                if split.len() < 2 {
                    continue;
                } else {
                    println!("Adding {} minutes", split[1]);
                }
            }
            _ => {println!("unrecognized command \"{}\"", input);}
        }
    }
}