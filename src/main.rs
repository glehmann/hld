#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;

mod cli;
mod cli_logger;
mod hld;

use std::io;
use structopt::StructOpt;

fn main() {
    let args = cli::Config::from_args();
    cli_logger::init(args.log_level).unwrap();

    if let Some(shell) = args.completion {
        cli::Config::clap().gen_completions_to("hld", shell, &mut io::stdout());
        std::process::exit(0);
    }

    if let Some(parallel) = args.parallel {
        debug!("using {} threads at most", parallel);
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallel)
            .build_global()
            .unwrap();
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
    let files = hld::glob_to_files(&file_globs).unwrap();
    let caches = hld::glob_to_files(&cache_globs).unwrap();
    trace!("files: {:?}", files);
    trace!("caches: {:?}", caches);
    if let Err(err) = hld::hardlink_deduplicate(
        &files,
        &caches,
        args.dry_run,
        &cache_path,
        args.clear_cache,
        args.strategy,
    ) {
        error!("{}", err);
        std::process::exit(1);
    }
}
