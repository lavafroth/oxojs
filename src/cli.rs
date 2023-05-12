use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input file containing all URLs
    #[arg()]
    pub input: Option<PathBuf>,

    /// User-Agent to send in requests
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Filepath to write results to
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Number of concurrent workers to spawn
    #[arg(short, long, default_value_t = 4)]
    pub concurrency: usize,

    /// Timeout (in seconds) for the client
    #[arg(short, long, default_value_t = 15)]
    pub timeout: u64,
}

pub fn args() -> Args {
    Args::parse()
}
