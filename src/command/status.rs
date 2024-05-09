use std::{collections::BTreeMap, fs};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    database::{Database, FlatTree},
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
        let mut flat_workspace = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&workspace.workspace, &mut flat_workspace);

        let mut index = Index::new(root_path.join(".rgit").join("index"));
        index.load()?;
        let mut flat_index = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&index.entries, &mut flat_index);

        let tracked_deleted_files = flat_commit_tree
            .entries
            .iter()
            .filter(|(path, _)| {
                !flat_index.entries.contains_key(*path)
                    && flat_commit_tree.entries.contains_key(*path)
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        let untracked_files = self.untracked_files(&flat_workspace, &flat_index, &flat_commit_tree);
        write_to_stdout("Untracked files:")?;
        for (file, _) in untracked_files {
            println!("{}", file.red());
        }

        let staged_files = self.tracked_files(&flat_workspace, &flat_index, &flat_commit_tree);

        write_to_stdout("Changes to be committed:")?;
        for (file, status) in staged_files.clone() {
            println!("{}: {}", status.green(), file.green());
        }

        let modified_files = flat_commit_tree
            .entries
            .iter()
            .filter_map(|(path, _)| {
                // if there's no key in the workspace, then this will be a deleted file
                if !flat_workspace.entries.contains_key(path) {
                    if flat_index.entries.contains_key(path) {
                        return Some((path.clone(), "deleted"));
                    }
                    return None;
                }
                let workspace_entry = flat_workspace.entries.get(path).unwrap();
                let workspace_entry_content = fs::read_to_string(&workspace_entry.path).unwrap();

                if !flat_index.entries.contains_key(path) {
                    return None;
                }

                let index_entry = flat_index.entries.get(path).unwrap();
                let index_entry_oid = index_entry.oid.as_ref().unwrap();
                let index_entry_content = decompress_content(&index_entry_oid).unwrap();
                if index_entry_content != workspace_entry_content {
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

    /// Returns a map of tracked files and their status
    /// If a file is in the index but not in the commit tree, it's a new file
    /// If a file is in the index and the commit tree but the content is different, it's a modified file
    /// If a file is in the commit tree but not in the index, it's a deleted file
    fn tracked_files(
        &self,
        workspace: &FlatIndex,
        index: &FlatIndex,
        commit_tree: &FlatTree,
    ) -> BTreeMap<String, String> {
        let mut tracked_files = BTreeMap::new();
        for (path, _) in index.entries.iter() {
            if !commit_tree.entries.contains_key(path) {
                tracked_files.insert(path.clone(), "new file".to_string());
                continue;
            }

            let commit_entry = commit_tree.entries.get(path).unwrap();
            let commit_entry_oid = commit_entry.oid.as_ref().unwrap();
            let commit_entry_content = decompress_content(&commit_entry_oid).unwrap();

            let index_entry = index.entries.get(path).unwrap();
            let index_entry_oid = index_entry.oid.as_ref().unwrap();
            let index_entry_content = decompress_content(&index_entry_oid).unwrap();

            if index_entry_content != commit_entry_content {
                tracked_files.insert(path.clone(), "modified".to_string());
            }
        }

        for (path, _) in commit_tree.entries.iter() {
            if !index.entries.contains_key(path) {
                tracked_files.insert(path.clone(), "deleted".to_string());
            }
        }
        tracked_files
    }

    /// Returns a map of untracked files
    /// If a file is in the workspace but not in the index, it's untracked and new
    /// If a file is in the index but not in the workspace, it's untracked and deleted
    fn untracked_files(
        &self,
        workspace: &FlatIndex,
        index: &FlatIndex,
        commit_tree: &FlatTree,
    ) -> BTreeMap<String, String> {
        let mut untracked_files = BTreeMap::new();
        for (path, _) in workspace.entries.iter() {
            if !index.entries.contains_key(path) {
                untracked_files.insert(path.clone(), "untracked".to_string());
            }
        }

        untracked_files
    }

    fn modified_files(
        &self,
        workspace: &FlatIndex,
        index: &FlatIndex,
        commit_tree: &FlatTree,
    ) -> BTreeMap<String, String> {
        let mut modified_files = BTreeMap::new();

        for (path, _) in commit_tree.entries.iter() {
            if !workspace.entries.contains_key(path) {
                if index.entries.contains_key(path) {
                    modified_files.insert(path.clone(), "deleted".to_string());
                }
                continue;
            }

            let workspace_entry = workspace.entries.get(path).unwrap();
            let workspace_entry_content = fs::read_to_string(&workspace_entry.path).unwrap();

            if !index.entries.contains_key(path) {
                continue;
            }

            let index_entry = index.entries.get(path).unwrap();
            let index_entry_oid = index_entry.oid.as_ref().unwrap();
            let index_entry_content = decompress_content(&index_entry_oid).unwrap();
            if index_entry_content != workspace_entry_content {
                modified_files.insert(path.clone(), "modified".to_string());
            }
        }

        modified_files
    }
}
