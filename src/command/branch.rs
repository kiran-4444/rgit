use anyhow::Result;
use clap::Parser;

use crate::{
    refs::Refs,
    utils::{get_root_path, write_to_stdout},
};

#[derive(Debug, Parser, PartialEq, Eq)]
#[clap(author, version, about = "A simple git branch CLI tool")]
pub struct BranchCMD {
    #[arg(short, long)]
    delete: Option<String>,

    #[arg(required = false)]
    name: Option<String>,

    #[arg(long)]
    list: bool,
}

impl BranchCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path);
        if self.list {
            self.list_branches()?;
        } else if let Some(name) = self.name {
            let output = format!("Create branch: {}", &name);
            write_to_stdout(&output)?;
            refs.create_branch(&name)?;
        } else if let Some(name) = self.delete {
            println!("Delete branch: {}", name);
        } else {
            self.list_branches()?;
        }
        Ok(())
    }

    fn list_branches(&self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path);
        let branches = refs.list_branches()?;
        let current_branch = refs.get_branch_name();
        for branch in branches {
            if branch == current_branch {
                write_to_stdout(&format!("* {}", branch))?;
            } else {
                write_to_stdout(&branch)?;
            }
        }
        Ok(())
    }
}
