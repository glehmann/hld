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
    #[arg(short, long = "cache")]
    pub caches: Vec<String>,

    /// Cache file
    #[arg(short = 'C', long)]
    pub cache_path: Option<PathBuf>,

    /// Clear the cache file
    #[arg(long)]
    pub clear_cache: bool,

    /// Recursively find the files in the provided paths
    #[arg(short, long)]
    pub recursive: bool,

    /// Don't modify anything on the disk
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// The linking strategy to use - either hardlink, symlink or reflink
    #[arg(short, long, default_value_t = Strategy::HardLink)]
    pub strategy: Strategy,

    /// Parallelism level
    #[arg(short = 'j', long)]
    pub parallel: Option<usize>,

    /// Log level
    #[arg(short = 'l', long, default_value = "info")]
    pub log_level: log::Level,

    /// Generate the completion code for this shell
    #[arg(long)]
    pub completion: Option<Shell>,
}

impl Config {
    pub fn cache_path(self: &Self) -> PathBuf {
        let path = if let Some(ref path) = self.cache_path {
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
