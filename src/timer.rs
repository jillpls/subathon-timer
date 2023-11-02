use crate::Message;
use crate::Senders;
use crate::Settings;
use std::sync::mpsc::Receiver;

pub(crate) async fn timer(_settings: Settings, senders: Senders, receiver: Receiver<Message>) {
    println!("Starting timer...");
    let _ = senders.cli.send(Message::Empty);

    loop {
        let msg = receiver.recv().unwrap_or(Message::Empty);
        println!("{:?}", msg);
    }
}
