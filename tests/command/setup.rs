use anyhow::Result;
use assert_cmd::prelude::*;
use std::{fs::write, path::PathBuf, process::Command};

pub fn get_rgit_cmd() -> Command {
    Command::cargo_bin("r_git").expect("Failed to build binary")
}

pub fn get_git_cmd() -> Command {
    Command::new("git")
}

pub fn setup_rgit(path: &PathBuf) -> Result<()> {
    let mut cmd = Command::cargo_bin("r_git")?;
    cmd.current_dir(&path).arg("init").assert().success();
    let rgitignore_path = path.join(".rgitignore");
    write(rgitignore_path, ".git/").expect("Failed to write .rgitignore file");

    std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
    std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");
    Ok(())
}

pub fn setup_git(path: &PathBuf) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.current_dir(&path).arg("init").assert().success();
    let gitignore_path = path.join(".gitignore");
    write(gitignore_path, ".rgit/").expect("Failed to write .gitignore file");
    Ok(())
}
