extern crate structopt;
#[macro_use]
extern crate log;
extern crate fs2;
extern crate glob;
extern crate num_cpus;
extern crate rayon;
extern crate serde_json;
extern crate sha1;
#[macro_use]
extern crate maplit;
extern crate ansi_term;
extern crate atty;

mod cli;
mod cli_logger;
mod hld;

use structopt::StructOpt;

fn main() {
    let args = cli::Config::from_args();
    cli_logger::init(if args.verbose {
        log::Level::Info
    } else {
        log::Level::Warn
    })
    .unwrap();

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
    if let Err(err) = hld::hardlink_deduplicate(&files, &caches) {
        error!("{}", err);
        std::process::exit(1);
    }
}
