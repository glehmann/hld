use std::fs;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub trait TestPathChild {
    fn child<P>(&self, path: P) -> assert_fs::fixture::ChildPath
    where
        P: AsRef<std::path::Path>;
    fn mkdir_all(&self) -> std::io::Result<()>;
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

/// returns the inodes of the partition and of the file
pub fn inos(path: &Path) -> (u64, u64) {
    let metadata = fs::metadata(path).unwrap();
    (metadata.st_dev(), metadata.ino())
}
