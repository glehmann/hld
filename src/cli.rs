use crate::strategy::*;
use clap::Parser;
use clap_complete::Shell;
use directories::ProjectDirs;
use std::path::PathBuf;

/// Hard Link Deduplicator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Files to process
    #[arg(name = "FILE")]
    pub files: Vec<String>,

    /// Files to cache
    #[arg(short = 'c', long = "cache", number_of_values = 1)]
    pub caches: Vec<String>,

    /// Cache file
    #[arg(short = 'C', long = "cache-path")]
    pub cache_path_opt: Option<PathBuf>,

    /// Clear the cache file
    #[arg(long = "clear-cache")]
    pub clear_cache: bool,

    /// Recursively find the files in the provided paths
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Don't modify anything on the disk
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// The linking strategy to use - either hardlink, symlink or reflink
    #[arg(short = 's', long = "strategy", default_value = "hardlink")]
    pub strategy: Strategy,

    /// Parallelism level
    #[arg(short = 'j', long = "parallel")]
    pub parallel: Option<usize>,

    /// Log level
    #[arg(short = 'l', long = "log-level", default_value = "info")]
    pub log_level: log::Level,
    /// Generate the completion code for this shell
    #[arg(long = "completion")]
    pub completion: Option<Shell>,
}

impl Config {
    pub fn cache_path(self: &Self) -> PathBuf {
        let path = if let Some(ref path) = self.cache_path_opt {
            path.clone()
        } else {
            let mut path = ProjectDirs::from("com", "glehmann", "hld")
                .unwrap()
                .cache_dir()
                .to_path_buf();
            path.push("digests");
            path
        };
        debug!("cache path: {}", path.display());
        path
    }
}
