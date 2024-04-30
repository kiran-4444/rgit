use anyhow::Result;
use clap::Parser;

use crate::{index::Index, utils::get_root_path, workspace::WorkspaceTree};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let workspace = WorkspaceTree::new(Some(&root_path));
        let mut index = Index::new(root_path.join(".rgit").join("index"));
        index.load()?;
        dbg!(workspace.workspace);
        dbg!(index.entries);
        Ok(())
    }
}
