use log;
use std::path::PathBuf;
use structopt::StructOpt;

/// Hard Link Deduplicator
#[derive(StructOpt, Debug)]
pub struct Config {
    /// Files to process
    #[structopt(name = "FILE")]
    pub files: Vec<String>,

    /// Files to cache
    #[structopt(short = "-c", long = "cache", raw(number_of_values = "1"))]
    pub caches: Vec<String>,

    /// Cache file
    #[structopt(short = "-C", long = "cache-path", parse(from_os_str))]
    pub cache_path_opt: Option<PathBuf>,

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

impl Config {
    pub fn cache_path(self: &Self) -> PathBuf {
        let path = if let Some(ref path) = self.cache_path_opt {
            path.clone()
        } else {
            let mut path = app_dirs::app_dir(
                app_dirs::AppDataType::UserCache,
                &app_dirs::AppInfo {
                    name: "hld",
                    author: "glehmann",
                },
                "",
            )
            .unwrap();
            path.push("digests");
            path
        };
        debug!("cache path: {}", path.display());
        path
    }
}
