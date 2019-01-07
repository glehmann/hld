mod common;

use crate::common::*;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use lipsum::lipsum;
use predicates::prelude::*;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::string::ToString;

#[test]
fn empty_run() {
    hld!()
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));
}

#[test]
fn parallel() {
    hld!("--log-level", "debug", "--parallel", "5")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("debug: using 5 threads at most"));
}

#[test]
fn invalid_glob() {
    hld!("foua/[etsin")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "error: foua/[etsin: Pattern syntax error near position 5: invalid range pattern",
        ));
}

#[test]
fn deduplication() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));

    hld!(tmp.child("*.txt"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(common::inos(foo.path()), common::inos(bar.path()));
}

#[test]
fn dryrun() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
    cache_path.assert(predicate::path::missing());

    hld!(
        "--log-level",
        "debug",
        "--cache",
        foo,
        "--cache-path",
        cache_path,
        bar,
        "--dry-run"
    )
    .assert()
    .success()
    .stdout(predicate::str::is_empty())
    .stderr(predicate::str::contains(format!(
        "{} saved in the deduplication of 1 files",
        pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
    )));

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
    cache_path.assert(predicate::path::missing());
}

#[test]
fn unreadable_file() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    foo.write_str(&lorem_ipsum).unwrap();

    fs::set_permissions(foo.path(), Permissions::from_mode(0o000)).unwrap();

    hld!(foo)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "error: {}: Permission denied (os error 13)",
            foo.path().display()
        )));
}

#[test]
fn no_deduplication_different_files() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lipsum(100)).unwrap();
    bar.write_str(&lipsum(101)).unwrap();

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));

    hld!(tmp.child("*.txt"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
}

#[test]
fn no_deduplication_empty_files() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.touch().unwrap();
    bar.touch().unwrap();

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));

    hld!(tmp.child("*.txt"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "0 B saved in the deduplication of 0 files",
        ));

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
}

#[test]
fn deduplication_with_cache() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
    cache_path.assert(predicate::path::missing());

    // first warm up the cache
    hld!(
        "--log-level",
        "debug",
        "--cache",
        foo,
        "--cache-path",
        cache_path
    )
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
    hld!(
        "--log-level",
        "debug",
        "--cache",
        foo,
        "--cache-path",
        cache_path,
        bar
    )
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

    assert_eq!(common::inos(foo.path()), common::inos(bar.path()));
}

#[test]
fn clear_cache() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    foo.write_str(&lorem_ipsum).unwrap();

    let cache_dir = assert_fs::TempDir::new().unwrap();
    let cache_path = cache_dir.child("digests");

    cache_path.assert(predicate::path::missing());

    // first warm up the cache
    hld!(
        "--log-level",
        "debug",
        "--cache",
        tmp.child("*"),
        "--cache-path",
        cache_path
    )
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
    hld!(
        "--log-level",
        "debug",
        "--cache",
        foo,
        "--cache-path",
        cache_path,
        "--clear-cache"
    )
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
fn recursive() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let subdir = tmp.child("hop").child("hop").child("hop");
    let foo = subdir.child("foo.txt");
    let bar = subdir.child("bar.txt");
    subdir.mkdir_all().unwrap();
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));

    hld!("--recursive", tmp)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(common::inos(foo.path()), common::inos(bar.path()));
}

#[test]
fn symlinking() {
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    let is_symlink = predicate::path::is_symlink();

    assert_ne!(common::inos(foo.path()), common::inos(bar.path()));
    assert!(!is_symlink.eval(foo.path()));
    assert!(!is_symlink.eval(bar.path()));

    hld!(tmp.child("*.txt"), "--strategy", "symlink")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(common::inos(foo.path()), common::inos(bar.path()));
    assert!(is_symlink.eval(foo.path()) ^ is_symlink.eval(bar.path()));
    assert_eq!(
        fs::read_link(foo.path()).unwrap_or(foo.path().to_path_buf()),
        fs::read_link(bar.path()).unwrap_or(bar.path().to_path_buf())
    );
}
