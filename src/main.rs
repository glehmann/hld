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
use structopt::StructOpt;

fn run() -> error::Result<()> {
    let args = cli::Config::from_args();
    cli_logger::init(args.log_level)?;

    if let Some(shell) = args.completion {
        cli::Config::clap().gen_completions_to("hld", shell, &mut io::stdout());
        std::process::exit(0);
    }

    if let Some(parallel) = args.parallel {
        debug!("using {} threads at most", parallel);
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallel)
            .build_global()?;
    }

    let cache_path = args.cache_path();
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
    let files = hld::glob_to_files(&file_globs)?;
    let caches = hld::glob_to_files(&cache_globs)?;
    trace!("files: {:?}", files);
    trace!("caches: {:?}", caches);
    hld::hardlink_deduplicate(
        &files,
        &caches,
        args.dry_run,
        &cache_path,
        args.clear_cache,
        args.strategy,
    )?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
