use color_eyre::{eyre::WrapErr, Help, Result};
use lazy_static::lazy_static;
use regex::Regex;
use soup::prelude::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use simple_logger::SimpleLogger;

use reqwest::{Client, Url};
use tokio::task;
mod cli;
mod client;

lazy_static! {
    static ref RE: Regex = Regex::new(r"[(\w./:)]*js").unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::args();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;
    color_eyre::install()?;

    let (jobs_tx, jobs_rx) = unbounded();
    let (results_tx, results_rx) = unbounded();

    for _ in 0..args.concurrency {
        let client = client::initialize(args.timeout, args.user_agent.clone())?;
        task::spawn(worker(client, jobs_rx.clone(), results_tx.clone()));
    }

    drop(results_tx);

    task::spawn(async move {
        let reader: Box<dyn BufRead> = match args.input {
            Some(path) => Box::new(BufReader::new(
                File::open(path).expect("Failed to open input file path for reading"),
            )),
            None => Box::new(io::stdin().lock()),
        };
        for line in reader.lines() {
            jobs_tx
                .send(
                    line.wrap_err("While trying to send jobs to workers")
                        .expect("Unable to read lines from input"),
                )
                .wrap_err("Unable to send jobs to workers")
                .unwrap();
        }
    });

    let mut handle: Box<dyn Write> = if let Some(filepath) = args.output {
        Box::new(
            File::create(&filepath)
                .wrap_err(format!(
                    "Failed to create file at path {}",
                    filepath.display()
                ))
                .suggestion("Try supplying a filename at a location where you can write to")?,
        )
    } else {
        Box::new(io::stdout())
    };

    for result in results_rx {
        writeln!(handle, "{}", result)
            .wrap_err("Failed to write to output file handle")
            .suggestion("Try supplying a filename at a location where you can write to")?;
    }
    Ok(())
}

pub async fn fetch(client: &Client, url: Url) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await
        .wrap_err("Failed to send request")?;

    Ok(response
        .text()
        .await
        .wrap_err("Failed to retrieve response text")?)
}

pub fn normalize_if_needed(js: String, url: &Url) -> String {
    if js.starts_with("http://") || js.starts_with("https://") {
        js
    } else {
        normalize(&js, url)
    }
}

pub fn normalize(js: &str, url: &Url) -> String {
    if js.starts_with("//") {
        format!("{}:{}", url.scheme(), js)
    } else if js.starts_with("/") {
        // The following unwraps must not panic as we have already
        // filtered out URLs which do not have a host string.
        format!("{}://{}{}", url.scheme(), url.host_str().unwrap(), js)
    } else {
        format!("{}://{}/{}", url.scheme(), url.host_str().unwrap(), js)
    }
}

pub async fn worker(client: Client, jobs: Receiver<String>, results: Sender<String>) {
    for job in jobs {
        let url = match Url::parse(&job) {
            Ok(url) => url,
            Err(e) => {
                log::warn!("URL will be ignored: Unable to parse: {job}: {e}");
                continue;
            }
        };

        if !url.has_host() {
            log::warn!("URL does not have host: URL ignored: {}", job);
            continue;
        }

        let text = match fetch(&client, url.clone()).await {
            Ok(text) => text,
            Err(e) => {
                log::warn!("Unable to fetch URL: {url}: {e}");
                continue;
            }
        };
        let soup = Soup::new(&text);
        let mut carved = vec![];

        for script in soup.tag("script").find_all() {
            if let Some(js) = script.get("src") {
                carved.push(normalize_if_needed(js, &url));
            }
            for js in RE.find_iter(&script.text()) {
                carved.push(normalize(js.as_str(), &url))
            }
        }

        for div in soup
            .tag("div")
            .find_all()
        {
            if let Some(js) = div.get("data-script-src") {
                carved.push(normalize_if_needed(js, &url));
            }
        }

        for js in carved.into_iter() {
            match results.send(js) {
                Ok(_) => {}
                Err(e) => {
                    log::warn!("Unable to send result from worker to main thread: {e}")
                }
            }
        }
    }
}
