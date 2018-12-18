extern crate structopt;
#[macro_use]
extern crate log;
extern crate glob;
extern crate loggerv;
extern crate serde_json;
extern crate sha1;

mod cli;
mod hld;

use structopt::StructOpt;

fn main() {
    let args = cli::Config::from_args();
    loggerv::Logger::new()
        .max_level(if args.verbose {
            log::Level::Info
        } else {
            log::Level::Warn
        })
        .module_path(false)
        .level(true)
        // .verbosity(args.occurrences_of("v"))
        // .line_numbers(true)
        .init()
        .unwrap();

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
        println!("{}", err);
        std::process::exit(1);
    }
}
