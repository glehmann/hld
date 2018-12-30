use log;
use structopt::StructOpt;

/// Hard Link Deduplicator
#[derive(StructOpt, Debug)]
pub struct Config {
    /// Files to process
    #[structopt(name = "FILE")]
    pub files: Vec<String>,

    /// Files to cache
    #[structopt(short = "-c", long = "cache", raw(number_of_values="1"))]
    pub caches: Vec<String>,

    /// Recursively find the files in the provided paths
    #[structopt(short = "r", long = "recursive")]
    pub recursive: bool,

    /// Don't modify anything on the disk
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,

    /// Parallelism level
    #[structopt(short = "j", long = "parallel")]
    pub parallel: Option<usize>,

    /// Log level
    #[structopt(short = "l", long = "log-level", default_value = "INFO")]
    pub log_level: log::Level,
}
