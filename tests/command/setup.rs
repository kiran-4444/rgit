use anyhow::Result;
use assert_cmd::prelude::*;
use std::{
    fs::{self, write},
    path::PathBuf,
    process::Command,
};
use tempdir::TempDir;

pub fn get_rgit_cmd() -> Command {
    Command::cargo_bin("rgit").expect("Failed to build binary")
}

pub fn get_git_cmd() -> Command {
    Command::new("git")
}

pub fn setup_rgit(path: &PathBuf) -> Result<()> {
    let mut cmd = Command::cargo_bin("rgit")?;
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

pub fn setup_fs(tempdir: &TempDir) -> Result<()> {
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
