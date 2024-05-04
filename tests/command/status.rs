use anyhow::Result;
use assert_cmd::prelude::*;
use std::fs::{self, write};
use tempdir::TempDir;

use crate::setup::{get_rgit_cmd, setup_rgit};

fn setup_fs(tempdir: &TempDir) -> Result<()> {
    // create the following directory structure:
    // ├── .rgitignore
    // ├── a.txt
    // ├── b.txt
    // ├── c.txt
    // ├── d.txt
    // ├── f
    // │   └── g.txt
    // ├── k
    // │   └── l
    // │       └── m
    // │           ├── o.txt
    // │           └── q.txt
    // ├── l.txt
    // └── run.sh

    let root = tempdir.path();
    fs::create_dir_all(root.join("f"))?;
    fs::create_dir_all(root.join("k/l/m"))?;
    write(root.join("a.txt"), "a")?;
    write(root.join("b.txt"), "b")?;
    write(root.join("c.txt"), "c")?;
    write(root.join("d.txt"), "d")?;
    write(root.join("f/g.txt"), "g")?;
    write(root.join("k/l/m/o.txt"), "o")?;
    write(root.join("k/l/m/q.txt"), "q")?;
    write(root.join("l.txt"), "l")?;
    write(root.join("run.sh"), "run")?;

    Ok(())
}

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
a.txt
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
a.txt";

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
a.txt
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
