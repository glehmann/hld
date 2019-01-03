use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use lipsum::lipsum;
use predicates::prelude::*;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn test_empty_run() {
    Command::main_binary()
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));
}

#[test]
fn test_help() {
    Command::main_binary()
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Hard Link Deduplicator").and(
                predicate::str::is_match(
                    r"-r, --recursive +Recursively find the files in the provided paths",
                )
                .unwrap(),
            ),
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_log_level() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "debug"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("debug: cache path:"));
}

#[test]
fn test_parallel() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "debug", "--parallel", "5"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("debug: using 5 threads at most"));
}

#[test]
fn test_version() {
    Command::main_binary()
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^hld \d+\.\d+\.\d+\n$").unwrap())
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_bad_option() {
    Command::main_binary()
        .unwrap()
        .arg("--foo")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::is_match(r"^error: .+ '--foo'")
                .unwrap()
                .and(predicate::str::contains("USAGE:")),
        );
}

#[test]
fn test_deduplication() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(inos(foo.path()), inos(bar.path()));

    Command::main_binary()
        .unwrap()
        .arg(tmp.child("*.txt").path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(inos(foo.path()), inos(bar.path()));
}

#[test]
fn test_dryrun() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    assert_ne!(inos(foo.path()), inos(bar.path()));
    cache_path.assert(predicate::path::missing());

    Command::main_binary()
        .unwrap()
        .args(&[
            "--log-level",
            "debug",
            "--cache",
            &foo.path().display().to_string(),
            "--cache-path",
            &cache_path.path().display().to_string(),
            &bar.path().display().to_string(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_ne!(inos(foo.path()), inos(bar.path()));
    cache_path.assert(predicate::path::missing());
}

#[test]
fn test_unreadable_file() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    foo.write_str(&lorem_ipsum).unwrap();

    fs::set_permissions(foo.path(), Permissions::from_mode(0o000)).unwrap();

    Command::main_binary()
        .unwrap()
        .arg(foo.path())
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "error: {}: Permission denied (os error 13)",
            foo.path().display()
        )));
}

#[test]
fn test_no_deduplication_different_files() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lipsum(100)).unwrap();
    bar.write_str(&lipsum(101)).unwrap();

    assert_ne!(inos(foo.path()), inos(bar.path()));

    Command::main_binary()
        .unwrap()
        .arg(tmp.child("*.txt").path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));

    assert_ne!(inos(foo.path()), inos(bar.path()));
}

#[test]
fn test_no_deduplication_empty_files() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.touch().unwrap();
    bar.touch().unwrap();

    assert_ne!(inos(foo.path()), inos(bar.path()));

    Command::main_binary()
        .unwrap()
        .arg(tmp.child("*.txt").path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));

    assert_ne!(inos(foo.path()), inos(bar.path()));
}

#[test]
fn test_deduplication_with_cache() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    assert_ne!(inos(foo.path()), inos(bar.path()));
    cache_path.assert(predicate::path::missing());

    // first warm up the cache
    Command::main_binary()
        .unwrap()
        .args(&[
            "--log-level",
            "debug",
            "--cache",
            &tmp.child("foo.txt").path().display().to_string(),
            "--cache-path",
            &cache_path.path().display().to_string(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(format!(
                "debug: computing digest of {}",
                foo.path().display()
            ))
            .and(predicate::str::contains("debug: saving updated cache")),
        );

    cache_path.assert(predicate::path::exists());

    // then deduplicate
    Command::main_binary()
        .unwrap()
        .args(&[
            "--log-level",
            "debug",
            "--cache",
            &foo.path().display().to_string(),
            "--cache-path",
            &cache_path.path().display().to_string(),
            &bar.path().display().to_string(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(format!(
                "debug: hardlinking {} and {}",
                foo.path().display(),
                bar.path().display()
            ))
            .and(predicate::str::contains("debug: saving updated cache").not())
            .and(
                predicate::str::contains(format!(
                    "debug: computing digest of {}",
                    foo.path().display()
                ))
                .not(),
            )
            .and(predicate::str::contains(format!(
                "debug: computing digest of {}",
                bar.path().display()
            ))),
        );

    assert_eq!(inos(foo.path()), inos(bar.path()));
}

#[test]
fn test_clear_cache() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    foo.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    cache_path.assert(predicate::path::missing());

    // first warm up the cache
    Command::main_binary()
        .unwrap()
        .args(&[
            "--log-level",
            "debug",
            "--cache",
            &tmp.child("foo.txt").path().display().to_string(),
            "--cache-path",
            &cache_path.path().display().to_string(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(format!(
                "debug: computing digest of {}",
                foo.path().display()
            ))
            .and(predicate::str::contains("debug: saving updated cache")),
        );

    cache_path.assert(predicate::path::exists());

    // check that the digest is recomputed with --clear-cache
    Command::main_binary()
        .unwrap()
        .args(&[
            "--log-level",
            "debug",
            "--cache",
            &tmp.child("foo.txt").path().display().to_string(),
            "--cache-path",
            &cache_path.path().display().to_string(),
            "--clear-cache",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(format!(
                "debug: computing digest of {}",
                foo.path().display()
            ))
            .and(predicate::str::contains("debug: saving updated cache")),
        );
}

#[test]
fn test_completion() {
    for shell in &["bash", "fish", "zsh"] {
        Command::main_binary()
            .unwrap()
            .args(&["--completion", shell])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not())
            .stderr(predicate::str::is_empty());
    }
}

#[test]
fn test_recursive() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let subdir = tmp.child("hop").child("hop").child("hop");
    let foo = subdir.child("foo.txt");
    let bar = subdir.child("bar.txt");
    subdir.mkdir_all().unwrap();
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(inos(foo.path()), inos(bar.path()));

    Command::main_binary()
        .unwrap()
        .args(&["--recursive", &tmp.path().display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(inos(foo.path()), inos(bar.path()));
}

// utility functions from here

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
fn inos(path: &Path) -> (u64, u64) {
    let metadata = fs::metadata(path).unwrap();
    (metadata.st_dev(), metadata.ino())
}
