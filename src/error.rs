use glob::PatternError;
use snafu::prelude::*;
use std::io;
use std::path::PathBuf;
use std::result;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
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

/// Extension trait for `io::Result`.
pub trait IOResultExt<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.context(PathIoSnafu { path })
    }
}

/// Extension trait for glob Result.
pub trait GlobResultExt<T> {
    fn glob_ctx<S: Into<String>>(self, glob: S) -> Result<T>;
}

impl<T> GlobResultExt<T> for result::Result<T, PatternError> {
    fn glob_ctx<S: Into<String>>(self, glob: S) -> Result<T> {
        self.context(GlobPatternSnafu { glob })
    }
}
