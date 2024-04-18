use anyhow::Result;
use clap::Parser;

use crate::{
    database::{Blob, Database},
    index::Index,
    utils::get_root_path,
    workspace::{Addable, WorkspaceTree},
};

#[derive(Parser, Debug, PartialEq)]
pub struct AddCMD {
    #[clap(required = true)]
    files: Vec<String>,
}

impl AddCMD {
    pub fn run(&self) -> Result<()> {
        let current_dir = get_root_path()?;
        let git_path = current_dir.join(".rgit");

        let mut workspace = WorkspaceTree::new(Some(&current_dir));
        let database = Database::new(git_path.join("objects"));
        let mut index = Index::new(git_path.join("index"));

        for file in &self.files {
            self.add_file(&file, &mut workspace, &database, &mut index)?;
        }

        Ok(())
    }

    fn add_file(
        &self,
        file: &str,
        workspace: &mut WorkspaceTree,
        database: &Database,
        index: &mut Index,
    ) -> Result<()> {
        let root_path = get_root_path()?;
        let files = match file {
            "." => WorkspaceTree::list_files(&root_path),
            _ => WorkspaceTree::list_files(&root_path.join(file)),
        };

        match index.load_for_update()? {
            true => {
                for entry in &files {
                    let raw_data = workspace.read_file(&entry.path)?;
                    let data = unsafe { std::str::from_utf8_unchecked(&raw_data) };
                    let mut blob = Blob::new(data.to_owned());
                    database.store(&mut blob)?;
                    let oid = blob.oid.expect("failed to get oid");
                    dbg!(oid.clone());
                    index.add(&entry, oid);
                }
                // should not worry about adding empty directories
                if files.len() > 0 {
                    index.write_updates()?;
                }
            }
            false => {
                anyhow::bail!("Failed to hold index for update");
            }
        }

        Ok(())
    }
}
