mod common;

use assert_cmd::prelude::*;
use predicates::prelude::predicate::str::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help() {
    hld!("--help")
        .success()
        .stdout(contains("Hard Link Deduplicator").and(
            is_match(r"-r, --recursive +Recursively find the files in the provided paths").unwrap(),
        ))
        .stderr(is_empty());
}

#[test]
fn version() {
    hld!("--version")
        .success()
        .stdout(is_match(r"^hld \d+\.\d+\.\d+\n$").unwrap())
        .stderr(is_empty());
}

#[test]
fn bad_option() {
    hld!("--foo").failure().stdout(is_empty()).stderr(
        is_match(r"^error: .+ '--foo'")
            .unwrap()
            .and(contains("Usage:")),
    );
}

#[test]
fn bad_strategy() {
    hld!("--strategy", "dumb")
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            r"invalid value 'dumb' for '--strategy <STRATEGY>'",
        ));
}

#[test]
fn log_level_error() {
    hld!("--log-level", "error")
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn log_level_info() {
    hld!("--log-level", "info")
        .success()
        .stdout(is_empty())
        .stderr(
            contains(" saved in the deduplication of ")
                .and(contains("trace: ").not())
                .and(contains("debug: ").not())
                .and(contains("info: ").not()),
        );
}

#[test]
fn log_level_debug() {
    hld!("--log-level", "debug")
        .success()
        .stdout(is_empty())
        .stderr(
            contains(" saved in the deduplication of ")
                .and(contains("trace: ").not())
                .and(contains("debug: "))
                .and(contains("info: ").not()),
        );
}

#[test]
fn log_level_trace() {
    hld!("--log-level", "trace")
        .success()
        .stdout(is_empty())
        .stderr(
            contains(" saved in the deduplication of ")
                .and(contains("trace: "))
                .and(contains("debug: "))
                .and(contains("info: ")),
        );
}

#[test]
fn completion() {
    for shell in &["bash", "fish", "zsh"] {
        hld!("--completion", shell)
            .success()
            .stdout(is_empty().not())
            .stderr(is_empty());
    }
}
