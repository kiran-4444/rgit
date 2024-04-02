use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{
    utils::{get_root_path},
    workspace::Workspace,
    workspace_tree::WorkspaceTree,
};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let workspace = Workspace::new(PathBuf::from("."));
        let root_path = get_root_path()?;
        let files = workspace.list_files(&root_path)?;
        let tree = WorkspaceTree::new(files);
        dbg!(tree);
        // for file in files {
        //     write_to_stdout(&format!("{}", file.name.display()))?;
        // }
        Ok(())
    }
}
