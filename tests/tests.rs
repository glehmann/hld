extern crate assert_cmd;
extern crate predicates;

use assert_cmd::prelude::*;
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
        .stdout(predicate::str::is_match(r"hld \d+\.\d+\.\d+").unwrap())
        .stderr(predicate::str::is_empty());
}
