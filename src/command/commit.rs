use std::path::PathBuf;
use std::{env, path::Path};

use clap::{arg, Parser};
use colored::Colorize;
use std::io::Write;

use crate::{database, utils::list_files};

use crate::objects::*;

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
        let object_store = git_path.join("objects");
        let db = database::Database::new(object_store);
        let files = list_files().unwrap();
        let files = files
            .iter()
            .filter(|file| PathBuf::from(file).is_file())
            .collect::<Vec<_>>();
        let entries = files
            .iter()
            .map(|file| {
                // check if the file is a directory
                let data = std::fs::read_to_string(file).unwrap();
                let mut blob = Blob::new(data.to_owned());
                db.store(&mut blob);
                Entry::new(file.to_string().to_owned(), blob.oid.unwrap().to_owned())
            })
            .collect::<Vec<Entry>>();

        let mut tree = Tree::new(entries);
        db.store(&mut tree);
        println!("{:?}", tree.oid);

        let (name, email) = self.get_config();
        let author = Author::new(&name, &email);
        println!("{:?}", author);

        let message = self.message.clone();
        let mut commit = Commit::new(tree.oid.unwrap(), author, &message);
        db.store(&mut commit);
        println!("{:?}", commit);

        self.update_head(&commit);
        println!("[(root-commit) {}] {}", commit.oid.unwrap(), message);
    }
    /// Get the author name and email from the environment variables
    fn get_config(&self) -> (String, String) {
        (
            env::var("RGIT_AUTHOR_NAME").unwrap(),
            env::var("RGIT_AUTHOR_EMAIL").unwrap(),
        )
    }

    fn update_head(&self, commit: &Commit) {
        let git_path = construct_git_path(&Path::new("."));
        let head = git_path.join("HEAD");
        let mut file = std::fs::File::create(head).unwrap();
        file.write(commit.oid.to_owned().unwrap().as_bytes())
            .unwrap();
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
