use anyhow::Result;

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use crate::{
    command::status::{modified_files, tracked_files},
    database::{Content, Database},
    diff::{EditType, Myres},
    index::{FlatIndex, Index},
    utils::{get_root_path, is_binary_file},
    workspace::{File, WorkspaceTree},
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
        let b_mode = index_file.stat.mode;

        let null_path = PathBuf::from("/dev/null");

        let output = format!("diff --git {} {}", a_path.display(), b_path.display());

        println!("{}", output.bold());

        let output = format!("old file mode {}", b_mode.to_string().bold());
        println!("{}", output.bold());

        let output = format!(
            "index {}..{} {}",
            self.short_oid(&a_oid),
            self.short_oid(&b_oid),
            b_mode
        );
        println!("{}", output.bold());

        let output = format!("--- {}\n+++ {}", null_path.display(), b_path.display());
        println!("{}", output.bold());

        let index_entry_oid = index_file.oid.as_ref().unwrap();
        let index_entry_content_bytes = Content::parse(index_entry_oid)
            .expect("failed to get content")
            .body;

        if is_binary_file(&index_entry_content_bytes).unwrap() {
            println!("Binary files differ");
            return;
        }

        let index_entry_content =
            String::from_utf8(index_entry_content_bytes).expect("failed to parse content to utf8");
        // let index_entry_content = String::from_utf8(Content::parse(index_entry_oid).unwrap().body)
        //     .expect("failed to parse content");

        let diff = Myres::new("".to_string(), index_entry_content);
        diff.diff();
    }

    fn diff_file_deleted(&self, index_file: &File) {
        let a_oid = index_file
            .oid
            .clone()
            .expect("OID not found for index entry");
        let a_path = PathBuf::from("a").join(index_file.path.clone());
        let a_mode = index_file.stat.mode;

        let b_oid = "0".repeat(40);
        let b_path = PathBuf::from("b").join(index_file.path.clone());

        let null_path = PathBuf::from("/dev/null");

        let output = format!("diff --git {} {}", a_path.display(), b_path.display());
        println!("{}", output.bold());

        let output = format!("deleted file mode {}", a_mode.to_string().bold());
        println!("{}", output.bold());

        let output = format!(
            "index {}..{} {}",
            self.short_oid(&a_oid),
            self.short_oid(&b_oid),
            a_mode
        );
        println!("{}", output.bold());

        let output = format!("--- {}\n+++ {}", a_path.display(), null_path.display());
        println!("{}", output.bold());

        let index_entry_oid = index_file.oid.as_ref().unwrap();
        let index_entry_content_bytes = Content::parse(index_entry_oid)
            .expect("failed to get content")
            .body;

        if is_binary_file(&index_entry_content_bytes).unwrap() {
            println!("Binary files differ");
            return;
        }
        let index_entry_content =
            String::from_utf8(index_entry_content_bytes).expect("failed to parse content to utf8");

        let diff = Myres::new(index_entry_content, "".to_string());
        diff.diff();
    }

    fn diff_file_modified(&self, workspace_file: &File, index_file: &File) {
        let a_oid = index_file
            .oid
            .clone()
            .expect("OID not found for index entry");
        let a_path = PathBuf::from("a").join(index_file.path.clone());
        let a_mode = index_file.stat.mode;

        let b_oid = workspace_file
            .oid
            .clone()
            .expect("OID not found for workspace entry");
        let b_path = PathBuf::from("b").join(workspace_file.path.clone());
        let b_mode = workspace_file.stat.mode;

        let output = format!("diff --git {} {}", a_path.display(), b_path.display());

        println!("{}", output.bold());
        if a_mode != b_mode {
            println!("old mode {}", a_mode.to_string().bold());
            println!("new mode {}", b_mode.to_string().bold());
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

        let workspace_entry_content_bytes = fs::read(&workspace_file.path).unwrap();

        if is_binary_file(&workspace_entry_content_bytes).unwrap() {
            println!("Binary files differ");
            return;
        }

        let workspace_entry_content = String::from_utf8(workspace_entry_content_bytes)
            .expect("failed to parse content to utf8");

        let index_entry_oid = index_file.oid.as_ref().unwrap();
        let index_entry_content = String::from_utf8(
            Content::parse(index_entry_oid)
                .expect("failed to get content")
                .body,
        )
        .expect("failed to parse content to utf8");

        let diff = Myres::new(index_entry_content, workspace_entry_content);
        let hunks = diff.diff();

        for hunk in hunks {
            let (a_offset, b_offset) = hunk.header();

            let hunks_offsets = format!(
                "@@ -{} +{} @@",
                a_offset
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                b_offset
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );

            println!("{}", hunks_offsets.cyan());

            for edit in hunk.edits {
                match edit.edit_type {
                    EditType::Add => {
                        println!("{}", format!("+{}", edit.b_line.unwrap().line).green());
                    }
                    EditType::Remove => {
                        println!("{}", format!("-{}", edit.a_line.unwrap().line).red());
                    }
                    EditType::Equal => {
                        println!("{}", format!(" {}", edit.a_line.unwrap().line));
                    }
                }
            }
        }
    }

    fn short_oid(&self, oid: &str) -> String {
        oid.chars().take(7).collect()
    }
}
