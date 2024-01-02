#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;

mod cli;
mod cli_logger;
mod error;
mod hld;
mod strategy;

use std::io;

use clap::{CommandFactory, Parser};
use clap_complete::generate;

fn run() -> error::Result<()> {
    let args = cli::Config::parse();
    cli_logger::init(args.log_level)?;

    if let Some(shell) = args.completion {
        generate(shell, &mut cli::Config::command(), "hld", &mut io::stdout());
        std::process::exit(0);
    }

    if let Some(parallel) = args.parallel {
        debug!("using {} threads at most", parallel);
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallel)
            .build_global()?;
    }

    let file_globs = if args.recursive {
        args.files.iter().map(|d| format!("{}/**/*", d)).collect()
    } else {
        args.files.clone()
    };
    let cache_globs = if args.recursive {
        args.caches.iter().map(|d| format!("{}/**/*", d)).collect()
    } else {
        args.caches.clone()
    };
    let files = hld::glob_to_files(&file_globs)?;
    let caches = hld::glob_to_files(&cache_globs)?;
    trace!("files: {:?}", files);
    trace!("caches: {:?}", caches);
    hld::hardlink_deduplicate(&args, &files, &caches)?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
