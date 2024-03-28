use anyhow::Result;
use assert_cmd::prelude::*;
use std::fs::{self, read, write};
use tempdir::TempDir;

use crate::setup::{get_git_cmd, get_rgit_cmd, setup_git, setup_rgit};

#[test]
fn test_add_with_no_args() {
    let mut cmd = get_rgit_cmd();
    cmd.arg("add").assert().failure();
}

#[test]
fn test_add_with_one_file() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("file");
    write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("file")
        .assert()
        .success();

    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    setup_git(&temp_dir.path().to_path_buf())?;

    let mut git_cmd = get_git_cmd();
    git_cmd
        .current_dir(&temp_dir)
        .arg("add")
        .arg("file")
        .assert()
        .success();

    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");

    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}

#[test]
fn test_add_with_directory() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let dir_path = temp_dir.path().join("dir");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");

    let file_path = dir_path.join("file");
    write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("dir")
        .assert()
        .success();
    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    setup_git(&temp_dir.path().to_path_buf())?;
    let mut git_cmd = get_git_cmd();
    git_cmd
        .current_dir(&temp_dir)
        .arg("add")
        .arg("dir")
        .assert()
        .success();
    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");

    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}

#[test]
fn test_add_with_files_and_directories() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("file_1");
    write(&file_path, "file_1").expect("Failed to write file");

    let dir_path = temp_dir.path().join("dir_1");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");

    let file_path = dir_path.join("file_2");
    write(&file_path, "file_2").expect("Failed to write file");

    let dir_path = temp_dir.path().join("dir_2").join("dir_3");
    std::fs::create_dir_all(&dir_path).expect("Failed to create directory");

    let file_path = dir_path.join("file_3");
    write(&file_path, "file_3").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["dir_1", "dir_2", "file_1"])
        .assert()
        .success();
    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    setup_git(&temp_dir.path().to_path_buf())?;
    let mut git_cmd = get_git_cmd();
    git_cmd
        .current_dir(&temp_dir)
        .arg("add")
        .args(&["dir_1", "dir_2", "file_1"])
        .assert()
        .success();
    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");
    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}

#[test]
fn test_add_with_duplicate_entries() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("file");
    write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["file", "file"])
        .assert()
        .success();
    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    setup_git(&temp_dir.path().to_path_buf())?;
    let mut git_cmd = get_git_cmd();
    git_cmd
        .current_dir(&temp_dir)
        .arg("add")
        .args(&["file", "file"])
        .assert()
        .success();
    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");
    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}

#[test]
fn test_add_with_same_file_name_and_dir_name() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    // fs::create_dir("test_rgit").expect("Failed to create temp dir");
    // let temp_dir = PathBuf::from("test_rgit");
    setup_rgit(&temp_dir.path().to_path_buf())?;
    setup_git(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("file");
    write(&file_path, "Hello, World!").expect("Failed to write file");
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["file"])
        .assert()
        .success();

    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["file"])
        .assert()
        .success();
    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");
    assert_eq!(rgit_index_content, git_index_content);

    fs::remove_file(&file_path).expect("Failed to remove file");
    let dir_path = temp_dir.path().join("file");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");
    let file_path = dir_path.join("file");
    write(&file_path, "Hello, World!").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["file"])
        .assert()
        .success();

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .args(&["file"])
        .assert()
        .success();

    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");
    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");

    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}

#[test]
fn adding_empty_dirs_should_succeed_without_updating_index() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;

    let dir_path = temp_dir.path().join("dir");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("dir")
        .assert()
        .success();

    assert_eq!(false, temp_dir.path().join(".rgit/index").exists());

    Ok(())
}

#[test]
fn adding_file_with_dirname_should_pass() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_rgit(&temp_dir.path().to_path_buf())?;
    setup_git(&temp_dir.path().to_path_buf())?;

    let file_path = temp_dir.path().join("alice.txt");
    write(&file_path, "Alice").expect("Failed to write file");

    let dir_path = temp_dir.path().join("nested");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");

    let file_path = dir_path.join("bob.txt");
    write(&file_path, "Bob").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("nested/")
        .assert()
        .success();

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("nested/")
        .assert()
        .success();

    // remove the nested directory
    fs::remove_dir_all(&dir_path).expect("Failed to remove directory");
    let file_path = temp_dir.path().join("nested");
    write(&file_path, "nested").expect("Failed to write file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("nested")
        .assert()
        .success();

    let mut cmd = get_git_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("nested")
        .assert()
        .success();

    let rgit_index_content =
        read(temp_dir.path().join(".rgit/index")).expect("Failed to read index file");

    let git_index_content =
        read(temp_dir.path().join(".git/index")).expect("Failed to read index file");

    assert_eq!(rgit_index_content, git_index_content);

    Ok(())
}
