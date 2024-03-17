use anyhow::Result;
use assert_cmd::prelude::*;
use std::process::Command;
use tempdir::TempDir;

pub fn get_rgit_cmd() -> Command {
    Command::cargo_bin("r_git").expect("Failed to build binary")
}

pub fn get_git_cmd() -> Command {
    Command::new("git")
}

pub fn setup_rgit(path: &TempDir) -> Result<()> {
    let mut cmd = Command::cargo_bin("r_git")?;
    cmd.current_dir(&path).arg("init").assert().success();
    Ok(())
}

pub fn setup_git(path: &TempDir) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.current_dir(&path).arg("init").assert().success();
    Ok(())
}
