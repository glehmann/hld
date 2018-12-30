extern crate structopt;
#[macro_use]
extern crate log;
extern crate fs2;
extern crate glob;
extern crate num_cpus;
extern crate rayon;
#[macro_use]
extern crate maplit;
extern crate ansi_term;
extern crate atty;
extern crate bincode;
extern crate blake2_rfc;
extern crate custom_error;
extern crate app_dirs;


mod cli;
mod cli_logger;
mod hld;

use structopt::StructOpt;

fn main() {
    let args = cli::Config::from_args();
    cli_logger::init(args.log_level).unwrap();

    if let Some(parallel) = args.parallel {
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallel)
            .build_global()
            .unwrap();
    }
    let file_globs = if args.recursive {
        args.files.iter().map(|d| format!("{}/**/*", d)).collect()
    } else {
        args.files
    };
    let cache_globs = if args.recursive {
        args.caches.iter().map(|d| format!("{}/**/*", d)).collect()
    } else {
        args.caches
    };
    let files = hld::glob_to_files(&file_globs).unwrap();
    let caches = hld::glob_to_files(&cache_globs).unwrap();
    let mut cache_path = args.cache_path.or_else(|| app_dirs::app_dir(
        app_dirs::AppDataType::UserCache,
        &app_dirs::AppInfo {name: "hld", author: "glehmann"},
        "").ok()).unwrap();
    cache_path.push("digests");
    debug!("cache file: {}", cache_path.display());
    if let Err(err) = hld::hardlink_deduplicate(&files, &caches, args.dry_run, &cache_path) {
        error!("{}", err);
        std::process::exit(1);
    }
}
