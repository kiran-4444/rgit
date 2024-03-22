use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{utils::write_to_stdout, workspace::Workspace};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let workspace = Workspace::new(PathBuf::from("."));
        let files = workspace.list_files(std::env::current_dir()?)?;
        for file in files {
            write_to_stdout(&format!("{}", file.name.display()))?;
        }
        Ok(())
    }
}
