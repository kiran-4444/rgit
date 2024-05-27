use crate::setup::{get_git_cmd, get_rgit_cmd, setup_git, setup_rgit};
use anyhow::Result;
use assert_cmd::prelude::*;
use std::{fs, process::Command};
use tempdir::TempDir;

#[test]
fn test_commit_without_root_should_fail() {
    let mut cmd = Command::cargo_bin("rgit").expect("Failed to build binary");
    cmd.arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .failure();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Error: fatal: not a git repository (or any of the parent directories): .rgit\n"
    );
}

#[test]
fn test_commit_without_author_details_should_fail() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit")?;
    setup_rgit(&temp_dir.path().to_path_buf())?;
    setup_git(&temp_dir.path().to_path_buf())?;

    // remove any author details (if set by previous tests)
    std::env::remove_var("RGIT_AUTHOR_NAME");
    std::env::remove_var("RGIT_AUTHOR_EMAIL");

    let file_path = temp_dir.path().join("file");
    std::fs::write(&file_path, "Hello, World!")?;

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("file")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .failure();

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Error: failed to get author details\n"
    );

    Ok(())
}

#[test]
fn test_commit_with_single_file() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit")?;
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("file");
    std::fs::write(&file_path, "Hello, World!")?;

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("file")
        .assert()
        .success();

    std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
    std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let output = cmd.output().expect("Failed to run command");
    let output = String::from_utf8_lossy(&output.stdout);
    let commit_oid = output
        .trim()
        .split(" ")
        .next()
        .expect("Failed to get commit oid");

    fs::rename(
        temp_dir.path().join(".rgit/"),
        temp_dir.path().join(".git/"),
    )?;

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir)
        .arg("cat-file")
        .arg("-p")
        .arg(commit_oid)
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_commit_chaining() {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
    std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");

    let file_path = temp_dir.path().join("file");
    fs::write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("file")
        .assert()
        .success();
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let file_path = temp_dir.path().join("file2");
    fs::write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("file2")
        .assert()
        .success();
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Second commit")
        .assert()
        .success();

    fs::rename(
        temp_dir.path().join(".rgit/"),
        temp_dir.path().join(".git/"),
    )
    .expect("Failed to rename .rgit to .git");

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir).arg("log").assert().success();

    let output = cmd.output().expect("Failed to run command");
    let output = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.split("\n").count(), 12);
}
