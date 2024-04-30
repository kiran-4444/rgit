use std::fs;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    index::{FlatIndex, Index},
    utils::{decompress_content, get_root_path, write_to_stdout},
    workspace::WorkspaceTree,
};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
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

        dbg!(&flat_workspace);
        dbg!(&flat_index);

        let untracked_files = flat_workspace
            .entries
            .iter()
            .filter(|(path, _)| !flat_index.entries.contains_key(*path))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        println!("{}", "\nUntracked files:".red());
        for file in untracked_files {
            println!("{}", file.red());
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
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        println!("{}", "\nModified files:".red());
        for file in modified_files.clone() {
            println!("{}", file.red());
        }

        let tracked_files = flat_index
            .entries
            .iter()
            .filter(|(path, _)| {
                flat_workspace.entries.contains_key(*path) && !modified_files.contains(&*path)
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        println!("{}", "\nTracked files:".green());
        for file in tracked_files {
            println!("{}", file.green());
        }

        Ok(())
    }
}
