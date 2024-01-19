use std::fs;
use std::os::unix::fs::MetadataExt;

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

#[cfg(not(feature = "kcov"))]
#[macro_export]
macro_rules! hld {
    ( $( $v:expr ),* ) => (
        {
            let temp_vec: Vec<String> = vec![$($v.to_string(),)*];
            Command::cargo_bin("hld").unwrap().args(&temp_vec).assert()
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
