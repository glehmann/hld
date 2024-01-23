mod common;

use crate::common::*;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use lipsum::lipsum;
use predicates::prelude::predicate::path::*;
use predicates::prelude::predicate::str::*;
use predicates::prelude::*;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::string::ToString;

#[test]
fn empty_run() {
    let _cache_dir = setup_cache_dir();
    hld!()
        .success()
        .stdout(is_empty())
        .stderr(contains("0 B saved in the deduplication of 0 files"));
}

#[test]
fn parallel() {
    let _cache_dir = setup_cache_dir();
    hld!("--log-level", "debug", "--parallel", "5")
        .success()
        .stdout(is_empty())
        .stderr(contains("debug: using 5 threads at most"));
}

#[test]
fn invalid_glob() {
    hld!("foua/[etsin")
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            "error: foua/[etsin: Pattern syntax error near position 5: invalid range pattern",
        ));
}

#[test]
fn deduplication() {
    let _cache_dir = setup_cache_dir();
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(inos(&foo), inos(&bar));

    hld!(tmp.child("*.txt"))
        .success()
        .stdout(is_empty())
        .stderr(contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(inos(&foo), inos(&bar));
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

    assert_ne!(inos(&foo), inos(&bar));
    cache_path.assert(missing());

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
    .success()
    .stdout(is_empty())
    .stderr(contains(format!(
        "{} saved in the deduplication of 1 files",
        pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
    )));

    assert_ne!(inos(&foo), inos(&bar));
    cache_path.assert(missing());
}

#[test]
fn unreadable_file() {
    let _cache_dir = setup_cache_dir();
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    foo.write_str(&lorem_ipsum).unwrap();

    fs::set_permissions(foo.path(), Permissions::from_mode(0o000)).unwrap();

    hld!(foo)
        .failure()
        .stdout(is_empty())
        .stderr(contains(format!(
            "error: {}: Permission denied (os error 13)",
            foo.path().display()
        )));
}

#[test]
fn no_deduplication_different_files() {
    let _cache_dir = setup_cache_dir();
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lipsum(100)).unwrap();
    bar.write_str(&lipsum(101)).unwrap();

    assert_ne!(inos(&foo), inos(&bar));

    hld!(tmp.child("*.txt"))
        .success()
        .stdout(is_empty())
        .stderr(contains("0 B saved in the deduplication of 0 files"));

    assert_ne!(inos(&foo), inos(&bar));
}

#[test]
fn no_deduplication_empty_files() {
    let _cache_dir = setup_cache_dir();
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.touch().unwrap();
    bar.touch().unwrap();

    assert_ne!(inos(&foo), inos(&bar));

    hld!(tmp.child("*.txt"))
        .success()
        .stdout(is_empty())
        .stderr(contains("0 B saved in the deduplication of 0 files"));

    assert_ne!(inos(&foo), inos(&bar));
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

    assert_ne!(inos(&foo), inos(&bar));
    cache_path.assert(missing());

    // first warm up the cache
    hld!(
        "--log-level",
        "debug",
        "--cache",
        foo,
        "--cache-path",
        cache_path
    )
    .success()
    .stdout(is_empty())
    .stderr(
        contains(format!(
            "debug: computing digest of {}",
            foo.path().display()
        ))
        .and(contains("debug: saving updated cache")),
    );

    cache_path.assert(exists());

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
    .success()
    .stdout(is_empty())
    .stderr(
        contains(format!(
            "debug: hardlinking {} and {}",
            foo.path().display(),
            bar.path().display()
        ))
        .and(contains("debug: saving updated cache").not())
        .and(
            contains(format!(
                "debug: computing digest of {}",
                foo.path().display()
            ))
            .not(),
        )
        .and(contains(format!(
            "debug: computing digest of {}",
            bar.path().display()
        ))),
    );

    assert_eq!(inos(&foo), inos(&bar));
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

    cache_path.assert(missing());

    // first warm up the cache
    hld!(
        "--log-level",
        "debug",
        "--cache",
        tmp.child("*"),
        "--cache-path",
        cache_path
    )
    .success()
    .stdout(is_empty())
    .stderr(
        contains(format!(
            "debug: computing digest of {}",
            foo.path().display()
        ))
        .and(contains("debug: saving updated cache")),
    );

    cache_path.assert(exists());

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
    .success()
    .stdout(is_empty())
    .stderr(
        contains(format!(
            "debug: computing digest of {}",
            foo.path().display()
        ))
        .and(contains("debug: saving updated cache")),
    );
}

#[test]
fn recursive() {
    let _cache_dir = setup_cache_dir();
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let subdir = tmp.child("hop").child("hop").child("hop");
    let foo = subdir.child("foo.txt");
    let bar = subdir.child("bar.txt");
    subdir.mkdir_all().unwrap();
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(inos(&foo), inos(&bar));

    hld!("--recursive", tmp)
        .success()
        .stdout(is_empty())
        .stderr(contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(inos(&foo), inos(&bar));
}

#[test]
fn symlinking() {
    let _cache_dir = setup_cache_dir();
    let lorem_ipsum = lipsum(100);
    // set up the test dir
    let tmp = assert_fs::TempDir::new().unwrap();
    let foo = tmp.child("foo.txt");
    let bar = tmp.child("bar.txt");
    foo.write_str(&lorem_ipsum).unwrap();
    bar.write_str(&lorem_ipsum).unwrap();

    assert_ne!(inos(&foo), inos(&bar));
    foo.assert(is_symlink().not());
    bar.assert(is_symlink().not());

    hld!(tmp.child("*.txt"), "--strategy", "symlink")
        .success()
        .stdout(is_empty())
        .stderr(contains(format!(
            "{} saved in the deduplication of 1 files",
            pretty_bytes::converter::convert(lorem_ipsum.len() as f64)
        )));

    assert_eq!(inos(&foo), inos(&bar));
    assert!(is_symlink().eval(foo.path()) ^ is_symlink().eval(bar.path()));
    assert_eq!(
        fs::read_link(foo.path()).unwrap_or(foo.path().to_path_buf()),
        fs::read_link(bar.path()).unwrap_or(bar.path().to_path_buf())
    );
}
