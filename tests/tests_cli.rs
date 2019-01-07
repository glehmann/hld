use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

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
