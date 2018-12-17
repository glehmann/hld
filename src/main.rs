extern crate structopt;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate sha1;
extern crate walkdir;

mod cli;
mod hld;

use std::process;
use structopt::StructOpt;

// use cli::Config;use std::process;

fn main() {
    let args = cli::Config::from_args();
    // opt.verbose.setup_env_logger("hld");
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

    let files = if args.recursive {
        hld::dirs_to_files(&args.files)
    } else {
        args.files
    };
    if let Err(err) = hld::hardlink_deduplicate(&files) {
        println!("{}", err);
        process::exit(1);
    }
}
