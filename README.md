# Photometry, Data Engineering, and Rust

Building a FITS data pipeline to transform and analyze raw light curve data from the TESS observatory.

## Motivation

A key benefit of the Rust language is the ability to detect data races at compile time. This detail has major implications for the development of parallelizable data pipelines. While there is not an official FITS file library written in Rust (yet [recognized](https://fits.gsfc.nasa.gov/fits_libraries.html) by NASA), developing one will be a task for another repo, and is a future goal of the author. astro-rs is used here to deserialize and handle the HDUs. Tokio is used to handle asynchronous requests to the online TESS data store at [archive.stsci.edu/tess](https://archive.stsci.edu/tess/bulk_downloads/bulk_downloads_ffi-tp-lc-dv.html)

## Access

- `read_urls(file: impl AsRef<Path>) -> io::Result<Vec<String>>` to parse the curl commands from a given .sh file associated with a sector's worth of raw data.

    ```rust
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
    ```

- `get_fits(client: Arc<Client>, url: String) -> Result<FitsHeader, Box<dyn std::error::Error>>` to make the request to the raw data store.

    ```rust
    async fn get_fits(client: Arc<Client>, url: String) -> Result<FitsHeader, Box<dyn std::error::Error>> {
        let resp = client.get(&url).send().await?;
        let bytes = resp.bytes().await?.to_vec();
        let fits = FitsHeader::from_bytes(bytes);

        Ok(fits)
    }
    ```

- `dispatcher(urls: Vec<String>, senders: Vec<mpsc::Sender<String>>)` to act as a thread pool manager. This distributes the output of the .sh file parsing to the worker threads.

    ```rust
    async fn dispatcher(urls: Vec<String>, senders: Vec<mpsc::Sender<String>>) {
        for (i, url) in urls.into_iter().enumerate() {
            let sender = &senders[i % senders.len()];
            sender.send(url).await.unwrap();
        }

        for sender in senders {
            sender.send("done".into()).await.unwrap();
        }
    }
    ```

- `worker(client: Arc<Client>, mut receiver: mpsc::Receiver<String>)` This is where the work happens. The FITS data is requested, read to a byte buffer, and ultimately deserialized. There is a place left for transforming the data. Output is collected into the main thread.

    ```rust
    async fn worker(client: Arc<Client>, mut receiver: mpsc::Receiver<String>) {
        loop {
            match receiver.recv().await {
                Some(url) => {
                    let fits = get_fits(client.clone(), url).await;
                    match fits {
                        Ok(_) => {
                            // Transform FITS file here...
                            println!("Successfully processed FITS file");
                        },
                        Err(err) => println!("Error processing FITS file: {:?}", err),
                    }
                }
                None => break,
            }
        }
    }
    ```

## Transformation

Under construction

## Conclusion

Under Construction
