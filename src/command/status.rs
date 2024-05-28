use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    database::{Content, Database, FlatTree},
    index::{FlatIndex, Index},
    utils::{get_root_path, write_to_stdout, write_to_stdout_color},
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

        let untracked_files = untracked_files(&flat_workspace, &flat_index);
        write_to_stdout("Untracked files:")?;
        for (file, _) in untracked_files {
            write_to_stdout_color(&file.red())?;
        }

        let staged_files = tracked_files(&flat_index, &flat_commit_tree);

        write_to_stdout("Changes to be committed:")?;
        for (file, status) in staged_files.clone() {
            let message = format!("{}: {}", status, file);
            write_to_stdout_color(&message.green())?;
        }

        let modified_files = modified_files(&flat_workspace, &flat_index, &flat_commit_tree);

        write_to_stdout("Changed not staged for commit:")?;
        for (file, status) in modified_files.clone() {
            let message = format!("{}: {}", status, file);
            write_to_stdout_color(&message.red())?;
        }

        Ok(())
    }
}

/// Returns a map of tracked files and their status
/// If a file is in the index but not in the commit tree, it's a new file
/// If a file is in the index and the commit tree but the content is different, it's a modified file
/// If a file is in the commit tree but not in the index, it's a deleted file
/// If a file is in the index but not in the workspace, it's deleted
pub fn tracked_files(index: &FlatIndex, commit_tree: &FlatTree) -> BTreeMap<String, String> {
    let mut tracked_files = BTreeMap::new();
    for (path, _) in index.entries.iter() {
        if !commit_tree.entries.contains_key(path) {
            tracked_files.insert(path.clone(), "new file".to_string());
            continue;
        }

        let commit_entry = commit_tree.entries.get(path).unwrap();
        let commit_entry_oid = commit_entry.oid.as_ref().unwrap();
        let object_store = PathBuf::from(".rgit/objects");
        let commit_entry_content = Content::parse(commit_entry_oid, object_store.clone())
            .unwrap()
            .body;

        let index_entry = index.entries.get(path).unwrap();
        let index_entry_oid = index_entry.oid.as_ref().unwrap();
        let index_entry_content = Content::parse(index_entry_oid, object_store).unwrap().body;

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
pub fn untracked_files(workspace: &FlatIndex, index: &FlatIndex) -> BTreeMap<String, String> {
    let mut untracked_files = BTreeMap::new();
    for (path, _) in workspace.entries.iter() {
        if !index.entries.contains_key(path) {
            untracked_files.insert(path.clone(), "untracked".to_string());
        }
    }

    untracked_files
}

/// Returns a map of modified files
/// If a file is in the workspace but not in the index, it's untracked and new
/// If a file is in the index but not in the workspace, it's untracked and deleted
/// If a file is in the index and the workspace but the content is different, it's a modified file
/// If a file is in the index and the workspace but the mode is different, it's a modified file
pub fn modified_files(
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
        let workspace_entry_content =
            unsafe { String::from_utf8_unchecked(fs::read(&workspace_entry.path).unwrap()) };

        if !index.entries.contains_key(path) {
            // If the file is in the workspace but not in the index, it's untracked
            continue;
        }

        let index_entry = index.entries.get(path).unwrap();
        let index_entry_oid = index_entry.oid.as_ref().unwrap();
        let object_store = PathBuf::from(".rgit/objects");
        let index_entry_content = unsafe {
            String::from_utf8_unchecked(Content::parse(index_entry_oid, object_store).unwrap().body)
        };
        if index_entry_content != workspace_entry_content
            || index_entry.stat.mode != workspace_entry.stat.mode
        {
            modified_files.insert(path.clone(), "modified".to_string());
        }
    }

    for (path, _) in index.entries.iter() {
        let index_entry = index.entries.get(path).unwrap();

        if !workspace.entries.contains_key(path) {
            // If the file is in the index but not in the workspace, it's untracked
            modified_files.insert(path.clone(), "deleted".to_string());
            continue;
        }

        // if the mode is different, it's a modified file
        if workspace.entries.contains_key(path) {
            let workspace_entry = workspace.entries.get(path).unwrap();
            if index_entry.stat.mode != workspace_entry.stat.mode {
                modified_files.insert(path.clone(), "modified".to_string());
            }
        }
    }

    modified_files
}
