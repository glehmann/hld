use std::fs;
use std::os::unix::fs::MetadataExt;

#[allow(dead_code)]
pub fn setup_cache_dir() -> assert_fs::TempDir {
    let cache = assert_fs::TempDir::new().unwrap();
    std::env::set_var("HLD_CACHE_PATH", cache.path().join("digests"));
    cache
}

pub trait TestPathChild {
    fn mkdir_all(&self) -> std::io::Result<()>;
}

pub trait TestToString {
    fn to_string(&self) -> String;
}

impl TestPathChild for assert_fs::fixture::ChildPath {
    fn mkdir_all(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path())
    }
}

impl TestToString for assert_fs::fixture::ChildPath {
    fn to_string(&self) -> String {
        self.path().display().to_string()
    }
}

impl TestToString for assert_fs::TempDir {
    fn to_string(&self) -> String {
        self.path().display().to_string()
    }
}

/// returns the inodes of the partition and of the file
#[allow(dead_code)]
pub fn inos(path: &assert_fs::fixture::ChildPath) -> (u64, u64) {
    let metadata = fs::metadata(path.path()).unwrap();
    (metadata.dev(), metadata.ino())
}

#[macro_export]
macro_rules! cargo_bin {
    ( $name:expr ) => {
        env!(concat!("CARGO_BIN_EXE_", $name))
    };
}

#[cfg(not(feature = "kcov"))]
#[macro_export]
macro_rules! hld {
    ( $( $v:expr ),* ) => (
        {
            let temp_vec: Vec<String> = vec![$($v.to_string(),)*];
            let runner_env = format!("CARGO_TARGET_{}_RUNNER", escargot::CURRENT_TARGET.replace("-", "_").to_uppercase());
            if let Ok(runner) = std::env::var(runner_env) {
                let runner_vec: Vec<_> = runner.split(" ").collect();
                Command::new(runner_vec[0]).args(&runner_vec[1..]).arg(cargo_bin!("hld")).args(&temp_vec).assert()
            } else {
                Command::new(cargo_bin!("hld")).args(&temp_vec).assert()
            }
        }
    );
}

#[cfg(feature = "kcov")]
#[macro_export]
macro_rules! hld {
    ( $( $v:expr ),* ) => (
        {
            let bin_path = escargot::CargoBuild::new()
                .current_release()
                .current_target()
                .run()
                .unwrap()
                .path()
                .to_path_buf();
            let coverage_dir = bin_path.parent().unwrap().join("coverage");
            std::fs::create_dir_all(&coverage_dir).unwrap();
            let temp_vec: Vec<String> = vec![
                "--include-pattern=/src".to_string(),
                "--exclude-pattern=/.cargo".to_string(),
                coverage_dir.display().to_string(),
                bin_path.display().to_string(),
                $($v.to_string(),)*
            ];
            Command::new("kcov").args(&temp_vec).assert()
        }
    );
}
