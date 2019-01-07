use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help() {
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
fn version() {
    Command::main_binary()
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^hld \d+\.\d+\.\d+\n$").unwrap())
        .stderr(predicate::str::is_empty());
}

#[test]
fn bad_option() {
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
fn log_level_error() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "error"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn log_level_info() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "info"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(" saved in the deduplication of ")
                .and(predicate::str::contains("trace: ").not())
                .and(predicate::str::contains("debug: ").not())
                .and(predicate::str::contains("info: ").not()),
        );
}

#[test]
fn log_level_debug() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "debug"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(" saved in the deduplication of ")
                .and(predicate::str::contains("trace: ").not())
                .and(predicate::str::contains("debug: "))
                .and(predicate::str::contains("info: ").not()),
        );
}

#[test]
fn log_level_trace() {
    Command::main_binary()
        .unwrap()
        .args(&["--log-level", "trace"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(" saved in the deduplication of ")
                .and(predicate::str::contains("trace: "))
                .and(predicate::str::contains("debug: "))
                .and(predicate::str::contains("info: ")),
        );
}

#[test]
fn completion() {
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
