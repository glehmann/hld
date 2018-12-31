extern crate assert_cmd;
extern crate assert_fs;
extern crate lipsum;
extern crate predicates;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use lipsum::lipsum;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_empty_run() {
    Command::main_binary()
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_help() {
    Command::main_binary()
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Hard Link Deduplicator").and(predicate::str::contains(
                "-r, --recursive    Recursively find the files in the provided paths",
            )),
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
        .stderr(predicate::str::contains("foo.txt").and(predicate::str::contains("bar.txt")));

    assert_eq!(inos(foo.path()), inos(bar.path()));
}

use std::fs;
use std::os::linux::fs::MetadataExt as LinuxMetadataExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// returns the inodes of the partition and of the file
fn inos(path: &Path) -> (u64, u64) {
    let metadata = fs::metadata(path).unwrap();
    (metadata.st_dev(), metadata.ino())
}
