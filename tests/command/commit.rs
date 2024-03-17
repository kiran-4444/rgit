use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_commit_without_root_should_fail() {
    let mut cmd = Command::cargo_bin("r_git").expect("Failed to build binary");
    cmd.arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .failure();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "fatal: not a git repository (or any of the parent directories): .rgit\n"
    );
}
