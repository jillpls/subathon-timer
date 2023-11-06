use crate::Message;
use crate::Senders;
use std::fs;
use std::sync::mpsc::Receiver;
use subathon_timer::EventCounts;

pub(crate) async fn timer(senders: Senders, receiver: Receiver<Message>) {
    println!("Starting timer...");
    let _ = senders.cli.send(Message::Empty);
    let mut event_counts = EventCounts {
        subs: 0,
        donations: 0.,
        bits: 0,
        channel_point_rewards : 0,
    };

    loop {
        if let Ok(s) = serde_json::to_string(&event_counts) {
            let _ = fs::write("timer.txt", s);
        }
        let msg = receiver.recv().unwrap_or(Message::Empty);
        println!("{:?}", msg);
        match msg {
            Message::AddBits(bits) => event_counts.bits += bits,
            Message::AddSub(sub) => {
                event_counts.subs += match sub {
                    2000 => 2,
                    3000 => 5,
                    _ => 1
                };
            },
            Message::AddDonation(don) => event_counts.donations += don,
            Message::AddChannelPointReward => event_counts.channel_point_rewards += 1,
            _ => {
                continue;
            }
        }
    }
}
