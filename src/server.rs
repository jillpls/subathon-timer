use serde::{Deserialize, Serialize};
use std::io;
use std::io::{stdin, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use warp::Filter;


mod api_server;
mod timer;
mod client;

#[derive(Clone)]
pub(crate) struct Senders {
    pub cli: Arc<Sender<Message>>,
    pub timer: Arc<Sender<Message>>,
}

unsafe impl Send for Senders {}

#[derive(Copy, Clone)]
pub(crate) struct Settings {
    kofi_ratio: f64,
    subscription_value: f64,
    bit_per_100_value: f64,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Empty,
    #[allow(unused)]
    String(String),
    AddTime(f64),
    #[allow(unused)]
    SubtractTime(f64),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EventLog {
    name: String,
    id: String,
    value: f64,
}
unsafe impl Send for Message {}

#[tokio::main]
async fn main() {
    let settings = Settings {
        kofi_ratio: 1.0,
        subscription_value: 4.0,
        bit_per_100_value: 1.0,
    };
    let (cli_tx, cli_rx) = mpsc::channel();
    let (timer_tx, timer_rx) = mpsc::channel();
    let senders_server = Senders {
        cli: Arc::new(cli_tx.clone()),
        timer: Arc::new(timer_tx.clone()),
    };
    let senders_timer = senders_server.clone();
    let server = tokio::spawn(async move {
        api_server::server(settings, [127, 0, 0, 1], 8080, senders_server).await;
    });
    let timer = tokio::spawn(async move {
        timer::timer(settings, senders_timer, timer_rx).await;
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
        let split = input.split(' ').collect::<Vec<_>>();
        if split.is_empty() {
            continue;
        }
        match split[0] {
            "stop" => {
                break;
            }
            "pause" => {
                println!("Pausing timer ...");
                let _ = timer_tx.send(Message::Empty);
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
