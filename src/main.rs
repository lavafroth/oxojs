use color_eyre::{
    eyre::{bail, WrapErr},
    Help, Result,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use log::{warn, LevelFilter::Info};
use regex::Regex;
use reqwest::{Client, Url};
use simple_logger::SimpleLogger;
use soup::prelude::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};
use tokio::task;
mod cli;
mod client;

lazy_static! {
    static ref RE: Regex = Regex::new(r"[\w./:]*?js").unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::args();
    SimpleLogger::new().with_level(Info).init()?;
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
    } else if js.starts_with('/') {
        // The following unwraps must not panic as we have already
        // filtered out URLs which do not have a host string.
        format!("{}://{}{}", url.scheme(), url.host_str().unwrap(), js)
    } else {
        format!("{}://{}/{}", url.scheme(), url.host_str().unwrap(), js)
    }
}

pub async fn scrape(client: &Client, job: &str, results: &Sender<String>) -> Result<()> {
    let url = Url::parse(job).wrap_err(format!("URL will be ignored: Unable to parse: {job}"))?;

    if !url.has_host() {
        bail!("URL does not have host: URL ignored: {job}");
    }

    let text = client
        .get(url.clone())
        .send()
        .await
        .wrap_err("Failed to send request")?
        .text()
        .await
        .wrap_err("Failed to retrieve response text")?;

    let soup = Soup::new(&text);

    let script_iter = soup.tag("script").find_all().flat_map(|script| {
        RE.find_iter(&script.text())
            .map(|js| normalize(js.as_str(), &url))
            .chain(script.get("src").map(|js| normalize_if_needed(js, &url)))
            .collect::<Vec<_>>()
    });

    let div_iter = soup.tag("div").find_all().filter_map(|div| {
        div.get("data-script-src")
            .map(|js| normalize_if_needed(js, &url))
    });

    for js in script_iter.chain(div_iter) {
        results.send(js).unwrap_or_else(|e| {
            warn!("Unable to send result from worker to main thread: {e}");
        });
    }
    Ok(())
}

pub async fn worker(client: Client, jobs: Receiver<String>, results: Sender<String>) {
    for job in jobs {
        scrape(&client, &job, &results)
            .await
            .unwrap_or_else(|e| warn!("{e}"))
    }
}
