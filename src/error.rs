use custom_error::custom_error;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::result;

custom_error! {pub Error
    PathIo {
        source: io::Error,
        path: PathBuf
    } = @{format!("{}: {}", path.display(), source)},
    // no need for this one for now, and not having it ensures we get a compilation error
    // when an io::Error is not properly converted to Error::PathIo
    // Io {source: io::Error} = "{source}",
    GlobPattern {source: glob::PatternError, glob: String} = "{glob}: {source}",
    Glob {source: glob::GlobError} = "{source}",
    Cache {src: bincode::Error} = "{src}",
    Strategy {name: String} = "unsupported '{}' strategy",
    Logger {source: log::SetLoggerError} = "{source}",
    ThreadPool {source: rayon::ThreadPoolBuildError} = "{source}",
}

/// Alias for a `Result` with the error type `hld::Error`.
pub type Result<T> = result::Result<T, Error>;

pub trait ToPathIOErr<T> {
    fn with_path(self: Self, path: &Path) -> Result<T>;
}

impl<T> ToPathIOErr<T> for io::Result<T> {
    fn with_path(self: Self, path: &Path) -> Result<T> {
        self.map_err(|e| Error::PathIo {
            source: e,
            path: path.to_path_buf(),
        })
    }
}

pub trait ToGlobPatternErr<T> {
    fn with_glob(self: Self, glob: &str) -> Result<T>;
}

impl<T> ToGlobPatternErr<T> for result::Result<T, glob::PatternError> {
    fn with_glob(self: Self, glob: &str) -> Result<T> {
        self.map_err(|e| Error::GlobPattern {
            source: e,
            glob: glob.to_owned(),
        })
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        Error::Cache { src: err }
    }
}
