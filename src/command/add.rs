use anyhow::Result;
use clap::Parser;

use crate::{
    database::{Blob, Database},
    index::{FlatIndex, Index},
    utils::get_root_path,
    workspace::WorkspaceTree,
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
        index.load()?;

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
        let mut flat_workspace = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&workspace.workspace, &mut flat_workspace);
        let mut flat_index = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&index.entries, &mut flat_index);

        let flat_commit_tree = database.read_head()?;

        let mut files = match file {
            "." => WorkspaceTree::list_files(&root_path),
            _ => {
                if root_path.join(file).exists() {
                    WorkspaceTree::list_files(&root_path.join(file))
                } else {
                    if !flat_workspace.entries.contains_key(file)
                        && flat_index.entries.contains_key(file)
                        && flat_commit_tree.entries.contains_key(file)
                    {
                        let vec = vec![flat_commit_tree.entries[file].clone()];
                        vec
                    } else {
                        anyhow::bail!("No such file: {}", file);
                    }
                }
            }
        };

        match index.load_for_update()? {
            true => {
                for entry in &mut files {
                    let key = entry.path.to_str().unwrap();
                    if !flat_workspace.entries.contains_key(key)
                        && flat_index.entries.contains_key(key)
                        && flat_commit_tree.entries.contains_key(key)
                    {
                        index.remove(entry);
                        continue;
                    }
                    let raw_data = workspace.read_file(&entry.path)?;
                    let data = unsafe { std::str::from_utf8_unchecked(&raw_data) };
                    let mut blob = Blob::new(data.to_owned());
                    database.store(&mut blob)?;
                    entry.oid = Some(blob.oid.unwrap().clone());
                    index.add(&entry);
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
