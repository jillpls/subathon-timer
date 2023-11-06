use std::{env, fs};
use std::io::{Read, stdin};
use std::path::Path;
use chrono::{Duration, Local};
use chrono::RoundingError::DurationExceedsLimit;
use subathon_timer::{EventCounts, Settings};

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

fn calculate_max_time_bonus(event_counts: &EventCounts, settings: &Settings) -> Duration {
    let mut result = Duration::zero();
    result = result + Duration::seconds(((event_counts.subs as f64 * settings.subscription_value)*60.).floor() as i64);
    result = result + Duration::seconds(((event_counts.donations * settings.kofi_ratio)*60.).floor() as i64);
    result = result + Duration::seconds(((event_counts.bits as f64 * settings.bit_per_100_value)*60./100.).floor() as i64);
    result = result + Duration::seconds(((event_counts.channel_point_rewards as f64 * settings.per_channel_point_reward)*60.).floor() as i64);
    result
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { panic!("No ip supplied"); }

    let url = args[1].clone();

    let (tx, rx) = std::sync::mpsc::channel();

    let response = tokio::spawn(async move {
        let url_txt = url.clone();
        loop {
            if let Ok(r) = reqwest::get(&url_txt).await {
                if let Ok(r) = r.text().await {
                    let _ = tx.send(r);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    let timer = tokio::spawn(async move {
        let mut max_time = chrono::Duration::hours(6);
        max_time = max_time + Duration::seconds(1);
        let mut event_counts = EventCounts::default();
        let mut bonus_time = Duration::zero();
        let mut local_bonus_time = Duration::zero();
        let mut time_expired =  if let Ok(s) = fs::read_to_string("time_expired.txt") {
                Duration::seconds(s.parse::<i64>().unwrap_or(0))
            } else {
                Duration::zero()
            };

        let timestamp = chrono::Local::now() - time_expired;

        let mut last_save = Local::now();

        loop {
            let r : Result<String, _> = rx.try_recv();
            if let Ok(r) = r {
                if let Ok(v) = serde_json::de::from_str::<EventCounts>(&r) {
                    event_counts = v;
                    bonus_time = calculate_max_time_bonus(&event_counts, &Settings::default());
                    local_bonus_time = get_local_bonus_time(&Settings::default());
                }
            }
            time_expired = Local::now() - timestamp;
            let fmtd = format(max_time + bonus_time + local_bonus_time - time_expired);
            let _ = fs::write("time_remaining.txt", fmtd);
            if Local::now()-last_save > Duration::seconds(5) {
                last_save = Local::now();
                let _ = fs::write("time_expired.txt", time_expired.num_seconds().to_string());
            }
        }
    });

    let display = tokio::spawn(async move {
    let mut current = fs::read_to_string("time_remaining.txt").unwrap_or(String::new());
    loop {
        let next = fs::read_to_string("time_remaining.txt").unwrap_or(String::new());
        if current != next {
            current = next;
            print!("\r{}", fs::read_to_string("time_remaining.txt").unwrap_or(String::new()));
        }
    }});

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    match &buffer {
        _ => println!("{}", buffer),
    }

    let _ = response.await;
    let _ = timer.await;
}

fn get_local_bonus_time(settings: &Settings) -> Duration {
    if let Ok(s) = fs::read_to_string("local_events.txt") {
        if let Ok(counts) = serde_json::from_str::<EventCounts>(&s) {
            return calculate_max_time_bonus(&counts, settings);
        }
    }
    Duration::zero()
}
