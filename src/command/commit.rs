use std::path::PathBuf;
use std::{env, path::Path};

use clap::{arg, Parser};
use colored::Colorize;

use crate::database;
use crate::refs;

use crate::objects::*;
use crate::workspace;

use super::init::{check_if_git_dir_exists, construct_git_path};

#[derive(Parser, Debug, PartialEq)]
pub struct CommitCMD {
    #[arg(short)]
    message: String,
}

impl CommitCMD {
    pub fn run(&self) {
        match check_if_git_dir_exists(&Path::new(".")) {
            false => {
                eprintln!(
                    "{}",
                    "fatal: not a git repository (or any of the parent directories): .rgit".red()
                );
                std::process::exit(1);
            }
            true => (),
        }
        let git_path = construct_git_path(&Path::new("."));
        let refs = refs::Refs::new(git_path.clone());
        let object_store = git_path.join("objects");
        let db = database::Database::new(object_store);
        let workspace = workspace::Workspace::new(PathBuf::from("."));
        let files = workspace.list_files().unwrap();
        let entries = files
            .iter()
            .map(|file| {
                // check if the file is a directory
                let file_name = file.0.as_ref().unwrap();
                let file_mode = file.1.as_str();
                let data = std::fs::read_to_string(file_name).unwrap();
                let mut blob = Blob::new(data.to_owned());
                db.store(&mut blob);
                Entry::new(
                    file_name.to_owned(),
                    blob.oid.unwrap().to_owned(),
                    file_mode.to_owned(),
                )
            })
            .collect::<Vec<Entry>>();

        let mut tree = Tree::new(entries);
        db.store(&mut tree);

        let (name, email) = self.get_config();
        let author = Author::new(&name, &email);

        let parent = refs.read_head();
        let message = self.message.clone();
        let mut commit = Commit::new(parent.to_owned(), tree.oid.unwrap(), author, &message);
        db.store(&mut commit);

        let commit_oid = commit.oid.clone().unwrap();
        refs.update_head(&commit_oid);

        match parent {
            Some(_) => {
                println!("{} {}", commit_oid, commit.message);
            }
            None => {
                println!("(root-commit) {} {}", commit_oid, commit.message);
            }
        }
    }
    /// Get the author name and email from the environment variables
    fn get_config(&self) -> (String, String) {
        (
            env::var("RGIT_AUTHOR_NAME").unwrap(),
            env::var("RGIT_AUTHOR_EMAIL").unwrap(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_retrieves_author_details_from_env_vars() {
        std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
        std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");

        let commit_cmd = CommitCMD {
            message: "".to_string(),
        };
        let (name, email) = commit_cmd.get_config();

        assert_eq!(name, "Test Author");
        assert_eq!(email, "test@example.com");
    }
}
