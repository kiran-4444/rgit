use std::fs;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{
    database::{Database, FlatTree, ParsedContent},
    index::{FlatIndex, Index},
    refs::Refs,
    utils::{decompress_content, get_root_path, write_to_stdout},
    workspace::{self, WorkspaceTree},
};

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path.clone());
        let parent = refs.read_head();
        let flat_commit_tree = match parent {
            Some(oid) => {
                let database = Database::new(git_path.join("objects"));
                let commit = database.read_object(&oid).unwrap();
                match commit {
                    ParsedContent::CommitContent(commit) => {
                        let tree_oid = commit.tree;
                        let tree = database.read_object(&tree_oid).unwrap();
                        if let ParsedContent::TreeContent(tree) = tree {
                            tree
                        } else {
                            panic!("should not happen")
                        }
                    }
                    _ => {
                        panic!("should not happen")
                    }
                }
            }
            None => FlatTree {
                entries: Default::default(),
            },
        };

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

        // dbg!(&flat_commit_tree, &flat_workspace, &flat_index);

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

        // get the files that are staged for commit
        let staged_files = flat_index
            .entries
            .iter()
            .filter(|(path, _)| {
                if !flat_commit_tree.entries.contains_key(*path) {
                    return true;
                }
                let commit_tree_entry = flat_commit_tree.entries.get(*path).unwrap();
                let index_entry = flat_index.entries.get(*path).unwrap();
                let index_oid = index_entry.oid.as_ref().unwrap();
                let index_decompressed_content = decompress_content(&index_oid).unwrap();
                let commit_tree_file_oid = commit_tree_entry.oid.as_ref().unwrap();
                let commit_tree_file_content = decompress_content(&commit_tree_file_oid).unwrap();
                index_decompressed_content != commit_tree_file_content
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("Changes to be committed:")?;
        for file in staged_files.clone() {
            println!("{}", file.green());
        }

        let modified_files = flat_commit_tree
            .entries
            .iter()
            .filter(|(path, _)| {
                let commit_tree_entry = flat_commit_tree.entries.get(*path).unwrap();
                let workspace_entry = flat_workspace.entries.get(*path).unwrap();
                let workspace_file_content = fs::read_to_string(&workspace_entry.path).unwrap();
                let commit_entry_oid = commit_tree_entry.oid.as_ref().unwrap();
                let decompressed_content = decompress_content(&commit_entry_oid).unwrap();
                workspace_file_content != decompressed_content && !staged_files.contains(path)
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        write_to_stdout("Changed not staged for commit:")?;
        for file in modified_files.clone() {
            println!("{}", file.red());
        }

        Ok(())
    }
}
