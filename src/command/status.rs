use std::{collections::BTreeMap, fs};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    database::Database,
    index::{FlatIndex, Index},
    utils::{decompress_content, get_root_path, write_to_stdout},
    workspace::WorkspaceTree,
};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let database = Database::new(git_path.join("objects"));
        let flat_commit_tree = database.read_head()?;

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

        let tracked_deleted_files = flat_commit_tree
            .entries
            .iter()
            .filter(|(path, _)| {
                !flat_workspace.entries.contains_key(*path)
                    && !flat_index.entries.contains_key(*path)
                    || flat_workspace.entries.contains_key(*path)
                        && !flat_index.entries.contains_key(*path)
                        && flat_commit_tree.entries.contains_key(*path)
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        let untracked_files = flat_workspace
            .entries
            .iter()
            .filter(|(path, _)| !flat_index.entries.contains_key(*path))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("Untracked files:")?;
        for file in untracked_files {
            println!("{}", file.red());
        }

        let staged_files = flat_index
            .entries
            .iter()
            .filter_map(|(path, _)| {
                // if there's no key in the commit tree, then this will be a new file
                if !flat_commit_tree.entries.contains_key(path) {
                    return Some((path.clone(), "new file"));
                }
                let commit_tree_entry = flat_commit_tree.entries.get(path).unwrap();
                let index_entry = flat_index.entries.get(path).unwrap();
                let index_oid = index_entry.oid.as_ref().unwrap();
                let index_decompressed_content = decompress_content(&index_oid).unwrap();
                let commit_tree_file_oid = commit_tree_entry.oid.as_ref().unwrap();
                let commit_tree_file_content = decompress_content(&commit_tree_file_oid).unwrap();

                if index_decompressed_content != commit_tree_file_content {
                    return Some((path.clone(), "modified"));
                }
                None
            })
            .collect::<BTreeMap<_, _>>();

        write_to_stdout("Changes to be committed:")?;
        for file in tracked_deleted_files.clone() {
            let prompt = format!("deleted: {}", file);
            println!("{}", prompt.green());
        }
        for (file, status) in staged_files.clone() {
            println!("{}: {}", status.green(), file.green());
        }

        let modified_files = flat_commit_tree
            .entries
            .iter()
            .filter_map(|(path, _)| {
                let commit_tree_entry = flat_commit_tree.entries.get(path).unwrap();
                // if there's no key in the workspace, then this will be a deleted file
                if !flat_workspace.entries.contains_key(path) {
                    if !tracked_deleted_files.contains(&path) {
                        return Some((path.clone(), "deleted"));
                    }
                    return None;
                }
                let workspace_entry = flat_workspace.entries.get(path).unwrap();
                let workspace_file_content = fs::read_to_string(&workspace_entry.path).unwrap();
                let commit_entry_oid = commit_tree_entry.oid.as_ref().unwrap();
                let decompressed_content = decompress_content(&commit_entry_oid).unwrap();
                if workspace_file_content != decompressed_content
                    && !staged_files.contains_key(path.as_str())
                    && !tracked_deleted_files.contains(&path)
                {
                    return Some((path.clone(), "modified"));
                }
                None
            })
            .collect::<BTreeMap<_, _>>();

        write_to_stdout("Changed not staged for commit:")?;
        for (file, status) in modified_files.clone() {
            println!("{}: {}", status.red(), file.red());
        }

        Ok(())
    }
}
