use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::{fmt::format, path::PathBuf};

use crate::{
    command::status::{modified_files, tracked_files, untracked_files},
    database::{Database, FlatTree},
    diff::Myres,
    index::{FlatIndex, Index},
    utils::{decompress_content, get_root_path, write_to_stdout, write_to_stdout_color},
    workspace::{self, File, WorkspaceTree},
};

#[derive(Parser, Debug, PartialEq)]
pub struct DiffCMD {
    #[clap(short, long)]
    pub cached: bool,
}

impl DiffCMD {
    pub fn run(&self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let database = Database::new(git_path.join("objects"));

        let mut index = Index::new(root_path.join(".rgit").join("index"));
        index.load()?;
        let mut flat_index = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&index.entries, &mut flat_index);

        let flat_commit_tree = database.read_head()?;

        let workspace = WorkspaceTree::new(Some(&root_path));
        let mut flat_workspace = FlatIndex {
            entries: Default::default(),
        };
        Index::flatten_entries(&workspace.workspace, &mut flat_workspace);

        if !self.cached {
            let modified_files = modified_files(&flat_workspace, &flat_index, &flat_commit_tree);
            for (file, status) in modified_files {
                if status == "modified" {
                    let workspace_file = flat_workspace.entries.get(&file).unwrap();
                    let index_file = flat_index.entries.get(&file).unwrap();
                    self.diff_file_modified(&workspace_file, &index_file);
                } else if status == "deleted" {
                    let index_file = flat_index.entries.get(&file).unwrap();
                    self.diff_file_deleted(&index_file);
                }
            }
            return Ok(());
        } else {
            let trackes_files = tracked_files(&flat_index, &flat_commit_tree);
            for (file, status) in trackes_files {
                if status == "modified" {
                    let index_file = flat_index.entries.get(&file).unwrap();
                    let commit_file = flat_commit_tree.entries.get(&file).unwrap();
                    self.diff_file_modified(&index_file, &commit_file);
                } else if status == "deleted" {
                    let index_file = flat_index.entries.get(&file).unwrap();
                    self.diff_file_deleted(&index_file);
                } else if status == "new file" {
                    let index_file = flat_index.entries.get(&file).unwrap();
                    self.diff_file_added(&index_file);
                }
            }
        }

        Ok(())
    }

    fn diff_file_added(&self, index_file: &File) {
        let a_oid = "0".repeat(40);
        let a_path = PathBuf::from("a").join(index_file.path.clone());

        let b_oid = index_file
            .oid
            .clone()
            .expect("OID not found for index entry");
        let b_path = PathBuf::from("b").join(index_file.path.clone());
        let b_mode = if index_file.stat.mode & 0o111 != 0 {
            "100755"
        } else {
            "100644"
        };

        let null_path = PathBuf::from("/dev/null");

        println!("diff --git {} {}", a_path.display(), b_path.display());
        println!("new file mode {}", b_mode);

        println!(
            "index {}..{}",
            self.short_oid(&a_oid),
            self.short_oid(&b_oid)
        );
        println!("--- {}", null_path.display());
        println!("+++ {}", b_path.display());
    }

    fn diff_file_deleted(&self, index_file: &File) {
        let a_oid = index_file
            .oid
            .clone()
            .expect("OID not found for index entry");
        let a_path = PathBuf::from("a").join(index_file.path.clone());
        let a_mode = if index_file.stat.mode & 0o111 != 0 {
            "100755"
        } else {
            "100644"
        };

        let b_oid = "0".repeat(40);
        let b_path = PathBuf::from("b").join(index_file.path.clone());

        let null_path = PathBuf::from("/dev/null");

        println!("diff --git {} {}", a_path.display(), b_path.display());
        println!("deleted file mode {}", a_mode);
        println!(
            "index {}..{}",
            self.short_oid(&a_oid),
            self.short_oid(&b_oid)
        );
        println!("--- {}", a_path.display());
        println!("+++ {}", null_path.display());
    }

    fn diff_file_modified(&self, workspace_file: &File, index_file: &File) {
        let a_oid = index_file
            .oid
            .clone()
            .expect("OID not found for index entry");
        let a_path = PathBuf::from("a").join(index_file.path.clone());
        let a_mode = if index_file.stat.mode & 0o111 != 0 {
            "100755"
        } else {
            "100644"
        };

        let b_oid = workspace_file
            .oid
            .clone()
            .expect("OID not found for workspace entry");
        let b_path = PathBuf::from("b").join(workspace_file.path.clone());
        let b_mode = if workspace_file.stat.mode & 0o111 != 0 {
            "100755"
        } else {
            "100644"
        };

        let output = format!("diff --git {} {}", a_path.display(), b_path.display());

        println!("{}", output.bold());
        if a_mode != b_mode {
            println!("old mode {}", a_mode.bold());
            println!("new mode {}", b_mode.bold());
        }
        if a_oid == b_oid {
            return;
        }

        if a_mode == b_mode {
            let output = format!(
                "index {}..{} {}",
                self.short_oid(&a_oid),
                self.short_oid(&b_oid),
                a_mode
            );
            println!("{}", output.bold());
        } else {
            let output = format!(
                "index {}..{}",
                self.short_oid(&a_oid),
                self.short_oid(&b_oid)
            );
            println!("{}", output.bold());
        }

        let output = format!("--- {}\n+++ {}", a_path.display(), b_path.display());
        println!("{}", output.bold());

        let workspace_entry_content = fs::read_to_string(&workspace_file.path).unwrap();

        let index_entry_oid = index_file.oid.as_ref().unwrap();
        let index_entry_content = decompress_content(&index_entry_oid).unwrap();

        let diff = Myres::new(index_entry_content, workspace_entry_content);
        diff.diff();
    }

    fn short_oid(&self, oid: &str) -> String {
        oid.chars().take(7).collect()
    }
}
