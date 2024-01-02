use snafu::prelude::*;
use std::io;
use std::path::PathBuf;
use std::result;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub), context(suffix(false)))]
pub enum Error {
    #[snafu(display("{}: {}", path.display(), source))]
    PathIo { source: io::Error, path: PathBuf },
    #[snafu(display("{glob}: {source}"))]
    GlobPattern {
        source: glob::PatternError,
        glob: String,
    },
    #[snafu(context(false))]
    Glob { source: glob::GlobError },
    #[snafu(context(false))]
    Cache { source: bincode::Error },
    #[snafu(display("unsupported '{name}' strategy"))]
    Strategy { name: String },
    #[snafu(context(false))]
    Logger { source: log::SetLoggerError },
    #[snafu(context(false))]
    ThreadPool { source: rayon::ThreadPoolBuildError },
}

/// Alias for a `Result` with the error type `hld::Error`.
pub type Result<T> = result::Result<T, Error>;
