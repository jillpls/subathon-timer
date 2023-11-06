use crate::serialize::FastExtract;
use crate::Message;
use crate::Senders;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use subathon_timer::Error;
use tokio::sync::Mutex;
use warp::http::HeaderMap;
use warp::Filter;

fn log_event(name: &str, id: &str, value: f64) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("events.log")?;

    let data_to_append = format!(
        "{{ \"name\":\"{}\", \"id\":\"{}\",\"value\":\"{}\"}}\n",
        name, id, value
    );

    file.write_all(data_to_append.as_bytes())?;
    Ok(())
}

async fn check_transaction_id(saved_ids: Arc<Mutex<HashSet<String>>>, id: &str) -> bool {
    let mut si = saved_ids.lock().await;
    if si.contains(id) {
        true
    } else {
        si.insert(id.to_string());
        false
    }
}

fn with_senders(sender: Senders) -> impl Filter<Extract = (Senders,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

async fn handle_twitch_event(
    _headers: HeaderMap,
    body: Value,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
) -> Result<String, Error> {
    if !body.is_object() {
        return Err(Error::cne("obj"));
    }
    if let Ok(c) = body.extract_object_key("challenge") {
        return c.extract_str();
    }
    let subscription = body.extract_object_key("subscription")?;
    let event_type = subscription.extract_object_str("type")?;
    let id = subscription.extract_object_str("id")?;
    let event = body.extract_object_key("event")?;
    if check_transaction_id(saved_ids, &id).await {
        return Ok("".to_string());
    }

    match event_type.as_str() {
        "channel.cheer" => {
            let amount: f64 = event.extract_object_f64("bits")?;
            let _ = senders.timer.send(Message::AddBits(amount.floor() as u64));
            let _ = log_event("twitch-cheer", &id, amount);
        }
        "channel.subscribe" => {
            let amount: f64 = event
                .extract_object_str("tier")?
                .parse()
                .map_err(|_| Error::ftp("str", "f64"))?;
            let _ = senders.timer.send(Message::AddSub(amount.floor() as u64));
            let _ = log_event("twitch-sub", &id, amount);
        }
        "channel.channel_points_custom_reward_redemption.add" => {
            let reward = event.extract_object_key("reward")?;
            let name = reward.extract_object_str("title")?;
            if name == "Subathon Bonus Minute" {
                let _ = senders.timer.send(Message::AddChannelPointReward);
                let _ = log_event("channel-point-reward", &id, 1.);
            } else {
                println!("{}", name);
            }
        }
        _ => {}
    }
    Ok("".to_string())
}

async fn handle_kofi_event(
    _headers: HeaderMap,
    body: HashMap<String, String>,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
) -> Result<String, Error> {
    let data = body.get("data").ok_or(Error::knf("data"))?;
    let data: Value = serde_json::from_str(data).map_err(|_| Error::ftp("str", "value"))?;
    let amount = data
        .extract_object_str("amount")?
        .parse()
        .map_err(|_| Error::ftp("str", "f64"))?;
    let transaction_id = data.extract_object_str("kofi_transaction_id")?;
    {
        let mut si = saved_ids.lock().await;
        if si.contains(&transaction_id) {
            println!("Duplicate transaction_id");
            return Ok("".to_string());
        }
        si.insert(transaction_id.to_string());
        let _r = log_event("kofi", &transaction_id, amount);
        let _ = senders.timer.send(Message::AddDonation(amount));
    }
    Ok("".to_string())
}

async fn handle_url_encoded(
    headers: HeaderMap,
    body: HashMap<String, String>,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
) -> Result<String, Error> {
    handle_kofi_event(headers, body, senders, saved_ids).await
}

async fn handle_json(
    headers: HeaderMap,
    body: Value,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
) -> Result<String, Error> {
    handle_twitch_event(headers, body, senders, saved_ids).await
}

pub(crate) async fn server(ip: [u8; 4], port: u16, senders: Senders) {
    println!("Starting api-server...");
    let saved_ids: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let saved_ids2 = saved_ids.clone();

    let routes = warp::path("channel")
        .map(|| "uwuwu")
        .or(warp::post()
            .and(warp::header::headers_cloned())
            .and(warp::body::json())
            .and(with_senders(senders.clone()))
            .and(warp::any().map(move || saved_ids.clone()))
            .and_then(
                move |headers, body, senders, saved_ids: Arc<Mutex<HashSet<String>>>| async move {
                    Ok::<_, Infallible>(
                        handle_json(headers, body, senders, saved_ids.clone()).await,
                    )
                },
            ))
        .or(warp::post()
            .and(warp::header::headers_cloned())
            .and(warp::body::form())
            .and(with_senders(senders.clone()))
            .and(warp::any().map(move || saved_ids2.clone()))
            .and_then(
                move |headers, body, senders, saved_ids: Arc<Mutex<HashSet<String>>>| async move {
                    Ok::<_, Infallible>(
                        handle_url_encoded(headers, body, senders, saved_ids.clone()).await,
                    )
                },
            ))
        .or(warp::get().map(|| {
            let r = fs::read("timer.txt");
            println!("{:?}", r);
            r.map_err(|_| { "failed to read file" } )
        }));
    let _ = senders.cli.send(Message::Empty);
    println!("{:?}", warp::serve(routes).run((ip, port)).await);
}
