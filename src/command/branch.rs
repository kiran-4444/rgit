use anyhow::Result;
use clap::Parser;

use crate::{
    refs::{parse_revision, Refs, Revision},
    utils::{get_root_path, write_to_stdout},
};

#[derive(Debug, Parser, PartialEq, Eq)]
pub struct BranchCMD {
    #[arg(short, long)]
    delete: Option<String>,

    #[arg(long)]
    list: bool,

    name: Vec<String>,
}

impl BranchCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path);
        if self.list {
            self.list_branches()?;
        } else if let Some(name) = self.delete {
            println!("Delete branch: {}", name);
        } else if !self.name.is_empty() {
            let name = &self.name;
            if name.len() == 1 {
                let name = &name[0];
                let output = format!("Create branch: {}", name);
                write_to_stdout(&output)?;
                let oid = refs.get_ref_content();
                refs.create_branch(name, &oid)?;
            } else if name.len() == 2 {
                let branch_name = &name[0];
                let rev = &name[1];
                let revision_object = parse_revision(rev);
                let oid = revision_object.resolve(&refs).expect("OID");
                refs.create_branch(branch_name, &oid)?;
            } else {
                let output = "Invalid branch format".to_string();
                write_to_stdout(&output)?;
            }
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
