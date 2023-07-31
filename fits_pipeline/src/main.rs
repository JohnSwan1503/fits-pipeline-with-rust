use astro_rs::fits::FitsHeader;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use reqwest::Client;

async fn read_urls(file: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    let mut urls = Vec::new();

    for line in reader.lines() {
        let line = line?;
        urls.push(line);
    }

    Ok(urls)
}

async fn get_fits(client: Arc<Client>, url: String) -> Result<FitsHeader, Box<dyn std::error::Error>> {
    let resp = client.get(&url).send().await?;
    let bytes = resp.bytes().await?.to_vec();
    let fits = FitsHeader::from_bytes(bytes);

    Ok(fits)
}

async fn worker(client: Arc<Client>, mut receiver: mpsc::Receiver<String>) {
    loop {
        match receiver.recv().await {
            Some(url) => {
                let fits = get_fits(client.clone(), url).await;
                match fits {
                    Ok(_) => {
                        // Do something with the FITS file
                        println!("Successfully processed FITS file");
                    },
                    Err(err) => println!("Error processing FITS file: {:?}", err),
                }
            }
            None => break,
        }
    }
}

async fn dispatcher(urls: Vec<String>, senders: Vec<mpsc::Sender<String>>) {
    for (i, url) in urls.into_iter().enumerate() {
        let sender = &senders[i % senders.len()];
        sender.send(url).await.unwrap();
    }

    // Close all channels so that the workers stop waiting for new jobs
    for sender in senders {
        sender.send("done".into()).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let urls = read_urls("data/").await?;
    let client = Arc::new(Client::new());

    let mut senders = Vec::new();
    let mut handles = Vec::new();

    for _ in 0..10 {
        let (sender, receiver) = mpsc::channel::<String>(100);
        senders.push(sender);
        let client = client.clone();
        let handle = task::spawn(worker(client, receiver));
        handles.push(handle);
    }

    task::spawn(dispatcher(urls, senders));

    // Wait for all workers to finish
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

