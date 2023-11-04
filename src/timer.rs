use crate::Message;
use crate::Senders;
use crate::Settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::mpsc::Receiver;
use subathon_timer::Timer;

pub(crate) async fn timer(_settings: Settings, senders: Senders, receiver: Receiver<Message>) {
    println!("Starting timer...");
    let _ = senders.cli.send(Message::Empty);
    let mut timer = Timer {
        subs: 0,
        donations: 0.,
        bits: 0,
    };

    loop {
        if let Ok(s) = serde_json::to_string(&timer) {
            let _ = fs::write("timer.txt", s);
        }
        let msg = receiver.recv().unwrap_or(Message::Empty);
        println!("{:?}", msg);
        match msg {
            Message::AddBits(bits) => timer.bits += bits,
            Message::AddSub(_sub) => timer.subs += 1,
            Message::AddDonation(don) => timer.donations += don,
            _ => {
                continue;
            }
        }
    }
}
