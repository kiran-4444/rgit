use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::write_to_stdout;

#[derive(Parser, Debug, PartialEq)]
pub struct InitCMD {
    name: Option<PathBuf>,
}

impl InitCMD {
    pub fn run(&self) -> Result<()> {
        match &self.name {
            Some(path) => initialize_git_dir(path)?,
            None => initialize_git_dir(Path::new("."))?,
        }
        Ok(())
    }
}

pub fn construct_git_path(path: &Path) -> Result<PathBuf> {
    let curr_dir = env::current_dir()?;

    let creation_path = if path == Path::new(".") {
        curr_dir
    } else {
        curr_dir.join(path)
    };

    Ok(creation_path.join(".rgit"))
}

pub fn initialize_git_dir(path: &Path) -> Result<()> {
    let creation_path = construct_git_path(path)?;
    // Remove the .rgit directory if it exists
    let if_exists = if creation_path.exists() {
        fs::remove_dir_all(&creation_path)?;
        true
    } else {
        false
    };

    // Create the .rgit directory with its parent directories
    fs::create_dir_all(&creation_path)?;

    // Create the objects directory
    fs::create_dir_all(&creation_path.join("objects"))?;

    // Create the refs directory
    fs::create_dir_all(&creation_path.join("refs/heads"))?;

    // Create the HEAD file
    fs::write(creation_path.join("HEAD"), "ref: refs/heads/master\n")?;

    // Give the user a nice message
    let console_output = if if_exists {
        format!(
            "Reinitialized empty Git repository in {}",
            creation_path.display()
        )
    } else {
        format!(
            "Initialized empty Git repository in {}",
            creation_path.display()
        )
    };

    write_to_stdout(&console_output)?;
    Ok(())
}
