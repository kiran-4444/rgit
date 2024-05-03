use std::fs;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    database::Database,
    index::{FlatIndex, Index},
    refs::Refs,
    utils::{decompress_content, get_root_path, write_to_stdout},
    workspace::WorkspaceTree,
};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {
    #[clap(short, long)]
    oid: String,
}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        if !self.oid.is_empty() {
            let root_path = get_root_path()?;
            let git_path = root_path.join(".rgit");
            let refs = Refs::new(git_path.clone());
            let parent = refs.read_head();
            let database = Database::new(git_path.join("objects"));
            let oid = self.oid;
            let tree = database.read_object(&oid)?;
            dbg!(tree);
        }
        let root_path = get_root_path()?;
        let workspace = WorkspaceTree::new(Some(&root_path));
        let mut index = Index::new(root_path.join(".rgit").join("index"));
        index.load()?;
        let mut flat_workspace = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&workspace.workspace, &mut flat_workspace);

        let mut flat_index = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&index.entries, &mut flat_index);

        let untracked_files = flat_workspace
            .entries
            .iter()
            .filter(|(path, _)| !flat_index.entries.contains_key(*path))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("\nUntracked files:")?;
        for file in untracked_files {
            println!("\t{}", file.red());
        }

        let modified_files = flat_index
            .entries
            .iter()
            .filter(|(path, _)| {
                let workspace_entry = flat_workspace.entries.get(*path).unwrap();
                let index_entry = flat_index.entries.get(*path).unwrap();
                let index_oid = index_entry.oid.as_ref().unwrap();
                let decompressed_content = decompress_content(&index_oid).unwrap();
                let workspace_file_content = fs::read_to_string(&workspace_entry.path).unwrap();
                workspace_entry.stat.mtime_nsec != index_entry.stat.mtime_nsec
                    && decompressed_content != workspace_file_content
                    || workspace_entry.stat.mtime_nsec == index_entry.stat.mtime_nsec
                        && workspace_entry.stat.mode != index_entry.stat.mode
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("\nChanged not staged for commit:")?;
        for file in modified_files.clone() {
            println!("\t{}", file.red());
        }

        // get the files that are staged for commit
        let staged_files = flat_index
            .entries
            .iter()
            .filter(|(path, _)| {
                let workspace_entry = flat_workspace.entries.get(*path).unwrap();
                let index_entry = flat_index.entries.get(*path).unwrap();
                let index_oid = index_entry.oid.as_ref().unwrap();
                let decompressed_content = decompress_content(&index_oid).unwrap();
                let workspace_file_content = fs::read_to_string(&workspace_entry.path).unwrap();
                workspace_entry.stat.mtime_nsec == index_entry.stat.mtime_nsec
                    && decompressed_content == workspace_file_content
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("\nChanges to be committed:")?;
        for file in staged_files {
            println!("\t{}", file.green());
        }

        Ok(())
    }
}
