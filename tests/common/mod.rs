use std::fs;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::MetadataExt;

pub trait TestPathChild {
    fn child<P>(&self, path: P) -> assert_fs::fixture::ChildPath
    where
        P: AsRef<std::path::Path>;
    fn mkdir_all(&self) -> std::io::Result<()>;
}

pub trait TestToString {
    fn to_string(&self) -> String;
}

impl TestPathChild for assert_fs::fixture::ChildPath {
    fn child<P>(&self, path: P) -> assert_fs::fixture::ChildPath
    where
        P: AsRef<std::path::Path>,
    {
        assert_fs::fixture::ChildPath::new(self.path().join(path))
    }
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
    (metadata.st_dev(), metadata.ino())
}

#[macro_export]
macro_rules! hld {
    ( $( $v:expr ),* ) => (
        {
            let temp_vec: Vec<String> = vec![$($v.to_string(),)*];
            Command::main_binary().unwrap().args(&temp_vec).assert()
        }
    );
}
