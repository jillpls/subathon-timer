use crate::Message;
use crate::Senders;
use crate::Settings;
use hyper::HeaderMap;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
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

fn with_senders(sender: Senders) -> impl Filter<Extract = (Senders,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

async fn handle_twitch_event(
    _headers: HeaderMap,
    body: Value,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
    settings: Settings,
) -> Result<String, String> {
    let body = if let Some(b) = body.as_object() {
        b
    } else {
        return Err("".to_string());
    };
    let subscription = body
        .get("subscription")
        .ok_or("no subscription found".to_string())?
        .as_object()
        .ok_or("subscription not an Object")?;
    let event_type = subscription
        .get("type")
        .ok_or("no type found".to_string())?
        .as_str()
        .ok_or("not a string".to_string())?;
    let id = subscription
        .get("id")
        .ok_or("no id found")?
        .as_str()
        .ok_or("not a string")?;
    let event = body
        .get("event")
        .ok_or("no subscription found".to_string())?
        .as_object()
        .ok_or("subscription not an Object")?;
    match event_type {
        "channel.cheer" => {
            let amount = event
                .get("bits")
                .ok_or("bits not found".to_string())?
                .as_number()
                .ok_or("not a number".to_string())?
                .as_f64()
                .ok_or("not a f64")?;
            let _ = senders
                .timer
                .send(Message::AddTime(amount / 100. * settings.bit_per_100_value));

            {
                let mut si = saved_ids.lock().await;
                if si.contains(id) {
                    println!("Duplicate transaction_id");
                    return Ok("".to_string());
                }
                si.insert(id.to_string());
            }
            let _ = log_event("twitch-cheer", id, amount);
        }
        "channel.subscribe" => {
            println!("{:?}", event.get("tier").unwrap());
            let amount = event
                .get("tier")
                .ok_or("bits not found".to_string())?
                .as_str()
                .ok_or("not a string".to_string())?
                .parse()
                .map_err(|_| "Parse error")?;
            let _ = senders.timer.send(Message::AddTime(
                amount / 1000. * settings.subscription_value,
            ));
            {
                let mut si = saved_ids.lock().await;
                if si.contains(id) {
                    println!("Duplicate transaction_id");
                    return Ok("".to_string());
                }
                si.insert(id.to_string());
            }
            let _ = log_event("twitch-sub", id, amount);
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
    settings: Settings,
) -> Result<String, String> {
    let data = body.get("data").ok_or("nani=".to_string())?;
    let data: Value = serde_json::from_str(data).or(Err("NANI???".to_string()))?;
    let data = data.as_object().ok_or("NANI????".to_string())?;
    let amount: f64 = data
        .get("amount")
        .ok_or("No amount".to_string())?
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let transaction_id = data
        .get("kofi_transaction_id")
        .ok_or("No transaction id".to_string())?
        .as_str()
        .unwrap();
    {
        let mut si = saved_ids.lock().await;
        if si.contains(transaction_id) {
            println!("Duplicate transaction_id");
            return Ok("".to_string());
        }
        si.insert(transaction_id.to_string());
        let _r = log_event("kofi", transaction_id, amount);
        let _ = senders
            .timer
            .send(Message::AddTime(settings.kofi_ratio * amount));
    }
    Ok("".to_string())
}

async fn handle_url_encoded(
    headers: HeaderMap,
    body: HashMap<String, String>,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
    settings: Settings,
) -> Result<String, String> {
    handle_kofi_event(headers, body, senders, saved_ids, settings).await
}

async fn handle_json(
    headers: HeaderMap,
    body: serde_json::Value,
    senders: Senders,
    saved_ids: Arc<Mutex<HashSet<String>>>,
    settings: Settings,
) -> Result<String, String> {
    handle_twitch_event(headers, body, senders, saved_ids, settings).await
}

pub(crate) async fn server(settings: Settings, ip: [u8; 4], port: u16, senders: Senders) {
    println!("Starting server...");
    let saved_ids: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let saved_ids2 = saved_ids.clone();

    let routes =
        warp::path("channel")
            .map(|| "uwuwu")
            .or(warp::post()
                .and(warp::header::headers_cloned())
                .and(warp::body::json())
                .and(with_senders(senders.clone()))
                .and(warp::any().map(move || saved_ids.clone()))
                .and(warp::any().map(move || settings))
                .and_then(
                    move |headers,
                          body,
                          senders,
                          saved_ids: Arc<Mutex<HashSet<String>>>,
                          settings| async move {
                        Ok::<_, Infallible>(
                            handle_json(headers, body, senders, saved_ids.clone(), settings).await,
                        )
                    },
                ))
            .or(warp::post()
                .and(warp::header::headers_cloned())
                .and(warp::body::form())
                .and(with_senders(senders.clone()))
                .and(warp::any().map(move || saved_ids2.clone()))
                .and(warp::any().map(move || settings))
                .and_then(
                    move |headers,
                          body,
                          senders,
                          saved_ids: Arc<Mutex<HashSet<String>>>,
                          settings| async move {
                        Ok::<_, Infallible>(
                            handle_url_encoded(headers, body, senders, saved_ids.clone(), settings)
                                .await,
                        )
                    },
                ))
            .or(warp::get().map(|| fs::read("timer.txt").map_err(|_| "oof")));
    let _ = senders.cli.send(Message::Empty);
    println!("{:?}", warp::serve(routes).run((ip, port)).await);
}
