use structopt::StructOpt;

/// Hard Link Deduplicator
#[derive(StructOpt, Debug)]
pub struct Config {
    /// Files to process
    #[structopt(name = "FILE")]
    pub files: Vec<String>,

    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Recursively find the files in the provided paths
    #[structopt(short = "r", long = "recursive")]
    pub recursive: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
