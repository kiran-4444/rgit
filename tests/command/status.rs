use anyhow::Result;
use assert_cmd::prelude::*;
use std::{
    fs::{self, write},
    os::unix::fs::PermissionsExt,
};
use tempdir::TempDir;

use crate::setup::{get_rgit_cmd, setup_fs, setup_rgit};

#[test]
fn test_fresh_status_command() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    let expected_output = "Untracked files:
.rgitignore
a.txt
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:";

    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn test_status_command_with_staged_files() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
new file: a.txt
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn test_status_command_with_modified_files() -> Result<()> {
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

    write(temp_dir.path().join("a.txt"), "modified")?;

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:
modified: a.txt";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
modified: a.txt
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("Second commit")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn test_status_command_with_deleted_files() {
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

    fs::remove_file(temp_dir.path().join("a.txt")).expect("Failed to remove file");

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:
deleted: a.txt";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
deleted: a.txt
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();
    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );
}

#[test]
fn test_modify_add_delete_add_status() -> Result<()> {
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

    write(temp_dir.path().join("a.txt"), "modified")?;

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    fs::remove_file(temp_dir.path().join("a.txt")).expect("Failed to remove file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
modified: a.txt
Changed not staged for commit:
deleted: a.txt";

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
deleted: a.txt
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn test_add_file_and_delete_it_status() -> Result<()> {
    let temp_dir = TempDir::new("test_rgit").expect("Failed to create temp dir");
    setup_fs(&temp_dir).expect("Failed to setup fs");
    setup_rgit(&temp_dir.path().to_path_buf()).expect("Failed to setup rgit");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    fs::remove_file(temp_dir.path().join("a.txt")).expect("Failed to remove file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
new file: a.txt
Changed not staged for commit:
deleted: a.txt";

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output,
    );

    Ok(())
}

#[test]
fn test_delete_add_create_new_status_add_status() -> Result<()> {
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

    fs::remove_file(temp_dir.path().join("a.txt")).expect("Failed to remove file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    write(temp_dir.path().join("a.txt"), "a")?;

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let expected_output = "Untracked files:
.rgitignore
a.txt
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
deleted: a.txt
Changed not staged for commit:";

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:";

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn test_modify_add_modify_status() -> Result<()> {
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

    write(temp_dir.path().join("a.txt"), "modified").expect("Failed to write to file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();

    write(temp_dir.path().join("a.txt"), "modified again").expect("Failed to write to file");

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
modified: a.txt
Changed not staged for commit:
modified: a.txt";

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}

#[test]
fn change_mode_should_show_as_modified() -> Result<()> {
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

    let mut perms = fs::metadata(temp_dir.path().join("a.txt"))?.permissions();
    perms.set_mode(0o777);
    fs::set_permissions(temp_dir.path().join("a.txt"), perms)?;

    let mut cmd = get_rgit_cmd();
    cmd.current_dir(&temp_dir).arg("status").assert().success();

    let expected_output = "Untracked files:
.rgitignore
b.txt
c.txt
d.txt
f/g.txt
k/l/m/o.txt
k/l/m/q.txt
l.txt
run.sh
Changes to be committed:
Changed not staged for commit:
modified: a.txt";

    let output = cmd.output().expect("Failed to run command");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );

    Ok(())
}
