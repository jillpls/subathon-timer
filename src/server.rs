use serde::{Deserialize, Serialize};
use std::io::{Write};
use std::sync::mpsc::{Sender};
use std::sync::{mpsc, Arc};
use warp::Filter;

mod api_server;
mod client;
mod serialize;
mod event_counts;

#[derive(Clone)]
pub(crate) struct Senders {
    pub cli: Arc<Sender<Message>>,
    pub timer: Arc<Sender<Message>>,
}

unsafe impl Send for Senders {}


#[derive(Clone, Debug)]
pub(crate) enum Message {
    Empty,
    String(String),
    AddDonation(f64),
    AddBits(u64),
    AddSub(u64),
    AddChannelPointReward,
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
    let (cli_tx, cli_rx) = mpsc::channel();
    let (timer_tx, timer_rx) = mpsc::channel();
    let senders_server = Senders {
        cli: Arc::new(cli_tx.clone()),
        timer: Arc::new(timer_tx.clone()),
    };
    let senders_timer = senders_server.clone();
    let server = tokio::spawn(async move {
        api_server::server([0, 0, 0, 0], 8080, senders_server).await;
    });
    let timer = tokio::spawn(async move {
        event_counts::timer(senders_timer, timer_rx).await;
    });

    server.await.expect("");
    timer.await.expect("");
}
