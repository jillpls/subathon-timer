use std::time::Duration;
use warp::hyper::{Uri, Client};
use warp::hyper::body::to_bytes;

#[tokio::main]
async fn main() {
    let url = "http://127.0.0.1:8080".parse::<Uri>().unwrap();

    let client = Client::new();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut response = Some(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let r = client.get(url).await.unwrap();
        let _ = tx.send(());
        r
    }));
    loop {
        if rx.try_recv().is_ok() {
            if response.is_some() {
                let r = response.take().unwrap();
                let response = r.await.unwrap();
                if !response.status().is_success() {
                    eprintln!("Request failed with status code: {}", response.status());
                }

                println!("{:?}", to_bytes(response.into_body()).await.unwrap());
            }
        }
    }
}
