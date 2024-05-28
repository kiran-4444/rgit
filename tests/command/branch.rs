use anyhow::Result;
use assert_cmd::prelude::*;
use std::fs::{self};
use tempdir::TempDir;

use crate::setup::{get_rgit_cmd, setup_fs, setup_rgit};

use rgit::refs::Refs;

#[test]
fn test_create_branch_with_invalid_branch_names() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let invalid_branch_names = vec![
        ".branch",
        "..hidden",
        "feature/.new",
        "release/.2024",
        "branch..name",
        "invalid..branch",
        "/new-branch",
        "/feature/test",
        "new-branch/",
        "feature/test/",
        "branch.lock",
        "release.lock",
        "branch@{",
        "feature@{test}",
        "branch with space",
        "branch:colon",
        "branch*star",
        "branch?question",
        "branch[bracket",
        "branch\\backslash",
        "branch^caret",
        "branch~tilde",
        "branch\u{7f}", // character with ASCII value 127
    ];

    for branch_name in invalid_branch_names {
        let mut cmd = get_rgit_cmd();
        cmd.current_dir(&temp_dir)
            .arg("branch")
            .arg(branch_name)
            .assert()
            .failure();

        let expected_output = format!("fatal: invalid branch name: {}\n", branch_name);
        let output = cmd.output().expect("Failed to run command");

        assert_eq!(String::from_utf8(output.stderr)?, expected_output);
    }

    Ok(())
}

#[test]
fn test_create_branch_should_fail_when_no_commit_on_master() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch")
        .assert()
        .failure();

    let expected_output = "fatal: Not a valid object name: 'HEAD'\n";
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(String::from_utf8(output.stderr)?, expected_output);

    Ok(())
}

#[test]
fn test_create_branch_command() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch")
        .assert()
        .success();

    let expected_output = "Create branch: test-branch\n";
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(String::from_utf8(output.stdout)?, expected_output);

    let branch_ref_path = temp_dir
        .path()
        .join(".rgit/")
        .join("refs/heads/test-branch");
    assert!(branch_ref_path.exists());

    // Check if the branch ref file has the correct content i.e. the commit hash
    let ref_content = fs::read_to_string(branch_ref_path)?;
    let head_ref_content = temp_dir.path().join(".rgit/refs/heads/master");
    dbg!(&head_ref_content);
    let head_ref_content = fs::read_to_string(head_ref_content)?;
    assert_eq!(ref_content, head_ref_content);

    Ok(())
}

#[test]
fn test_list_branches_command() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();
    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("--list")
        .assert()
        .success();

    let expected_output = "* master\ntest-branch\n";
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected_output);

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("branch").assert().success();

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected_output);

    Ok(())
}

#[test]
fn text_invalid_revisions() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let invalid_revisions = vec!["asdf", "HEAD^", "HEAD~", "HEAD~1", "HEAD~2", "HEAD~3"];

    for revision in invalid_revisions {
        let mut cmd = get_rgit_cmd();
        cmd.current_dir(&temp_dir)
            .arg("branch")
            .arg("test-branch")
            .arg(revision)
            .assert()
            .failure();

        let expected_output = format!("fatal: Not a valid object name: '{}'\n", revision);
        let output = cmd.output().expect("Failed to run command");

        assert_eq!(String::from_utf8(output.stderr)?, expected_output);
    }

    Ok(())
}

#[test]
fn test_revisions_in_branch_command() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("b.txt")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Second commit")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("c.txt")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Third commit")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch_1")
        .arg("@")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch_2")
        .arg("HEAD")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch_3")
        .arg("@^")
        .assert()
        .success();

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("branch")
        .arg("test-branch_4")
        .arg("@~2")
        .assert()
        .success();

    let refs = Refs::new(temp_dir.path().join(".rgit"));
    let all_commits = refs.get_all_commits()?;

    let test_branch_1_content = refs.get_specific_ref_content("test-branch_1");
    let test_branch_2_content = refs.get_specific_ref_content("test-branch_2");
    let test_branch_3_content = refs.get_specific_ref_content("test-branch_3");
    let test_branch_4_content = refs.get_specific_ref_content("test-branch_4");

    let head_oid = all_commits[0].oid.clone().unwrap();
    assert_eq!(test_branch_1_content, head_oid);
    assert_eq!(test_branch_2_content, head_oid);

    let head_parent_oid = all_commits[1].oid.clone().unwrap();
    assert_eq!(test_branch_3_content, head_parent_oid);

    let head_parent_parent_oid = all_commits[2].oid.clone().unwrap();
    assert_eq!(test_branch_4_content, head_parent_parent_oid);

    Ok(())
}
