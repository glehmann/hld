use std::io;
use std::path::PathBuf;
use std::result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{path}: {source}")]
    PathIo { source: io::Error, path: PathBuf },
    #[error("{glob}: {source}")]
    GlobPattern {
        source: glob::PatternError,
        glob: String,
    },
    #[error(transparent)]
    Glob(#[from] glob::GlobError),
    #[error(transparent)]
    Cache(#[from] bincode::Error),
    #[error(transparent)]
    Logger(#[from] log::SetLoggerError),
    #[error(transparent)]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),
}

/// Alias for a `Result` with the error type `hld::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Extension trait for `io::Result`.
pub trait IOResultExt<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.map_err(|source| Error::PathIo {
            source,
            path: path.into(),
        })
    }
}

/// Extension trait for glob Result.
pub trait GlobResultExt<T> {
    fn glob_ctx<S: Into<String>>(self, glob: S) -> Result<T>;
}

impl<T> GlobResultExt<T> for result::Result<T, glob::PatternError> {
    fn glob_ctx<S: Into<String>>(self, glob: S) -> Result<T> {
        self.map_err(|source| Error::GlobPattern {
            source,
            glob: glob.into(),
        })
    }
}
