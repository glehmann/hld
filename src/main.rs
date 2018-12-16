extern crate structopt;
extern crate sha1;

mod cli;
mod lib;

use std::path::PathBuf;
use structopt::StructOpt;
use std::process;

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
    let opt = Config::from_args();
    lib::hardlink_deduplicate(&opt.files, opt.verbose).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });
}
