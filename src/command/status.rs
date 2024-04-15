use anyhow::Result;
use clap::Parser;

use crate::{utils::get_root_path, workspace::WorkspaceTree};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let tree = WorkspaceTree::new(&root_path);
        dbg!(tree);
        // for file in files {
        //     write_to_stdout(&format!("{}", file.name.display()))?;
        // }
        Ok(())
    }
}
