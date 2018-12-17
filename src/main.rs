extern crate structopt;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate sha1;

mod cli;
mod hld;

use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

/// Hard Link Deduplicator
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Config {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}
// use cli::Config;use std::process;

fn main() {
    let args = Config::from_args();
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
    if let Err(err) = hld::hardlink_deduplicate(&args.files) {
        println!("{}", err);
        process::exit(1);
    }
}
