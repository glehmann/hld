use crate::strategy::*;
use clap::{Parser, ValueEnum};
use clap_complete::Shell;
use directories::ProjectDirs;
use std::path::PathBuf;
use strum::Display;

/// Hard Link Deduplicator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Files to process
    #[arg(name = "FILE")]
    pub files: Vec<String>,

    /// Files to cache
    #[arg(short, long = "cache")]
    pub caches: Vec<String>,

    /// Cache file
    #[arg(short = 'C', long, default_value = defaut_cache_path().into_os_string(), env = "HLD_CACHE_PATH")]
    pub cache_path: PathBuf,

    /// Clear the cache file
    #[arg(long)]
    pub clear_cache: bool,

    /// Recursively find the files in the provided paths
    #[arg(short, long, env = "HLD_RECURSIVE")]
    pub recursive: bool,

    /// Don't modify anything on the disk
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// The linking strategy to use - either hardlink, symlink or reflink
    #[arg(short, long, default_value_t = Strategy::HardLink, env = "HLD_STRATEGY")]
    pub strategy: Strategy,

    /// Parallelism level
    #[arg(short = 'j', long, env = "HLD_PARALLEL")]
    pub parallel: Option<usize>,

    /// Log level
    #[arg(short = 'l', long, default_value_t = Level::Info, env = "HLD_LOG_LEVEL")]
    pub log_level: Level,

    /// Generate the completion code for this shell
    #[arg(long)]
    pub completion: Option<Shell>,
}

#[derive(ValueEnum, Clone, Debug, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for log::Level {
    fn from(v: Level) -> log::Level {
        match v {
            Level::Trace => log::Level::Trace,
            Level::Debug => log::Level::Debug,
            Level::Info => log::Level::Info,
            Level::Warn => log::Level::Warn,
            Level::Error => log::Level::Error,
        }
    }
}

pub fn defaut_cache_path() -> PathBuf {
    let mut path = ProjectDirs::from("com", "glehmann", "hld")
        .unwrap()
        .cache_dir()
        .to_path_buf();
    path.push("digests");
    path
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Config::command().debug_assert()
}
