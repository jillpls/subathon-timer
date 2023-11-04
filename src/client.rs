use std::{env, fs};
use std::panic::panic_any;
use chrono::{Duration, Local};
use warp::hyper::body::{Bytes, to_bytes};
use warp::hyper::{Client, Uri};
use subathon_timer::Timer;

fn format(duration:  Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes= duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let mut result = "".to_string();
    if days > 0 {
        result.push_str(&days.to_string() );
        result.push(':');
    }
    result.push_str(&hours.to_string() );
    result.push(':');
    if minutes < 10 {result.push('0')}
    result.push_str(&minutes.to_string() );
    result.push(':');
    if seconds < 10 {result.push('0')}
    result.push_str(&seconds.to_string() );
    result
}

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { panic!("No ip supplied"); }

    let url = args[1].parse::<Uri>().unwrap();

    let client = Client::new();
    let (tx, rx) = std::sync::mpsc::channel();

    let response = tokio::spawn(async move {
        loop {
            let r = client.get(url.clone()).await;
            if let Ok(r) = r {
                if r.status().is_success() {
                    let _ = tx.send(r.into_body());
                }
            }
        }
    });

    let timer = tokio::spawn(async move {
        let mut max_time = chrono::Duration::hours(6);
        max_time = max_time + Duration::seconds(1);
        let mut timestamp = chrono::Local::now();

        loop {
            let r = rx.try_recv();
            if let Ok(r) = r {
                let r = to_bytes(r).await.unwrap_or(Bytes::new());
                if let Ok(v) = serde_json::de::from_str::<Timer>(&String::from_utf8_lossy(&r)) {
                    println!("{:?}", &v);
                }
            }
            let time_expired = Local::now() - timestamp;
            let fmtd = format(max_time - time_expired);
            let _ = std::fs::write("time_remaining.txt", fmtd);
        }
    });

    let mut current = fs::read_to_string("time_remaining.txt").unwrap_or(String::new());
    loop {
        let next = fs::read_to_string("time_remaining.txt").unwrap_or(String::new());
        if current != next {
            current = next;
            print!("\r{}", fs::read_to_string("time_remaining.txt").unwrap_or(String::new()));
        }
    }

    let _ = response.await;
    let _ = timer.await;
}
