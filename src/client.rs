use chrono::RoundingError::DurationExceedsLimit;
use chrono::{Duration, Local};
use std::fs::{read_to_string, OpenOptions};
use std::io::{stdin, Read, Write};
use std::path::Path;
use std::{env, fs};
use subathon_timer::{EventCounts, Settings};

fn format(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let mut result = "".to_string();
    if days > 0 {
        result.push_str(&days.to_string());
        result.push(':');
    }
    result.push_str(&hours.to_string());
    result.push(':');
    if minutes < 10 {
        result.push('0')
    }
    result.push_str(&minutes.to_string());
    result.push(':');
    if seconds < 10 {
        result.push('0')
    }
    result.push_str(&seconds.to_string());
    result
}

fn calculate_max_time_bonus(event_counts: &EventCounts, settings: &Settings) -> Duration {
    let mut result = Duration::zero();
    result = result
        + Duration::seconds(
            ((event_counts.subs as f64 * settings.subscription_value) * 60.).floor() as i64,
        );
    result = result
        + Duration::seconds(((event_counts.donations * settings.kofi_ratio) * 60.).floor() as i64);
    result = result
        + Duration::seconds(
            ((event_counts.bits as f64 * settings.bit_per_100_value) * 60. / 100.).floor() as i64,
        );
    result = result
        + Duration::seconds(
            ((event_counts.channel_point_rewards as f64 * settings.per_channel_point_reward) * 60.)
                .floor() as i64,
        );
    result
}

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("No ip supplied");
    }

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
        let mut time_expired = if let Ok(s) = fs::read_to_string("time_expired.txt") {
            Duration::seconds(s.parse::<i64>().unwrap_or(0))
        } else {
            Duration::zero()
        };

        let timestamp = chrono::Local::now() - time_expired;

        let mut last_save = Local::now();
        let mut local_event_counts = read_local_event_counts();
        let mut bonus_time =
            calculate_max_time_bonus(&(event_counts + local_event_counts), &Settings::default());

        loop {
            let r: Result<String, _> = rx.try_recv();
            if let Ok(r) = r {
                if let Ok(v) = serde_json::de::from_str::<EventCounts>(&r) {
                    event_counts = v;
                    local_event_counts = read_local_event_counts();
                    let combined = event_counts + local_event_counts;
                    truncate_write_to_file("subs.txt", &combined.subs.to_string());
                    truncate_write_to_file(
                        "fake_subs.txt",
                        &calculate_fake_subs(&combined, &Settings::default()).to_string(),
                    );
                    bonus_time = calculate_max_time_bonus(
                        &(event_counts + local_event_counts),
                        &Settings::default(),
                    );
                }
            }
            time_expired = Local::now() - timestamp;
            let mut time_remaining = max_time + bonus_time - time_expired;
            if time_remaining < Duration::zero() {
                time_remaining = Duration::zero();
            }
            let fmtd = format(time_remaining);
            truncate_write_to_file("time_remaining.txt", &fmtd);
            if Local::now() - last_save > Duration::seconds(5) {
                last_save = Local::now();
                let _ = fs::write("time_expired.txt", time_expired.num_seconds().to_string());
            }
        }
    });

    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    match &buffer {
        _ => println!("{}", buffer),
    }

    let _ = response.await;
    let _ = timer.await;
}

fn calculate_fake_subs(event_counts: &EventCounts, settings: &Settings) -> u64 {
    let mut result = event_counts.subs;
    result += (event_counts.channel_point_rewards as f64 * settings.per_channel_point_reward / 4.)
        .floor() as u64;
    result += (event_counts.bits as f64 * settings.bit_per_100_value / 4.).floor() as u64;
    result += (event_counts.donations * settings.kofi_ratio / 4.).floor() as u64;
    result
}

fn read_local_event_counts() -> EventCounts {
    if let Ok(s) = fs::read_to_string("local_events.txt") {
        if let Ok(counts) = serde_json::from_str::<EventCounts>(&s) {
            return counts;
        }
    }
    return EventCounts::default();
}

fn truncate_write_to_file(path: &str, content: &str) {
    let r = OpenOptions::new().write(true).open(path);
    if let Ok(mut f) = r {
        let _ = f.write_all(content.as_bytes());
    }
}
