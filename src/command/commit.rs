use anyhow::Result;
use clap::{arg, Parser};
use std::env;

use crate::{
    database::{Author, Commit, Database, Tree},
    index::Index,
    refs::Refs,
    utils::{get_root_path, write_to_stdout},
    workspace::FileOrDir,
};

#[derive(Parser, Debug, PartialEq)]
pub struct CommitCMD {
    #[arg(short)]
    message: String,
}

impl CommitCMD {
    pub fn run(&self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path.clone());
        let object_store = git_path.join("objects");
        let mut db = Database::new(object_store);
        let mut index = Index::new(git_path.join("index"));

        index.load()?;
        let mut root = Tree::new();
        root.build_from_index(&index);
        root.traverse(&mut db)?;
        db.store(&mut root)?;

        let (name, email) = self
            .get_config()
            .map_err(|_| anyhow::anyhow!("failed to get author details"))?;
        let author = Author::new(&name, &email);

        let parent = refs.read_head();
        let message = self.message.clone();
        let mut commit = Commit::new(
            parent.to_owned(),
            root.oid.expect("OID not found"),
            author,
            &message,
        );
        db.store(&mut commit)?;

        // update the index to store the tree cache

        let commit_oid = commit.oid.expect("Failed to get commit oid").clone();
        refs.update_head(&commit_oid)?;

        write_to_stdout(&format!("{} {}", commit_oid, commit.message))?;
        Ok(())
    }
    /// Get the author name and email from the environment variables
    fn get_config(&self) -> Result<(String, String)> {
        Ok((
            env::var("RGIT_AUTHOR_NAME")?,
            env::var("RGIT_AUTHOR_EMAIL")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_retrieves_author_details_from_env_vars() -> Result<()> {
        std::env::set_var("RGIT_AUTHOR_NAME", "Test Author");
        std::env::set_var("RGIT_AUTHOR_EMAIL", "test@example.com");

        let commit_cmd = CommitCMD {
            message: "".to_string(),
        };
        let (name, email) = commit_cmd.get_config()?;

        assert_eq!(name, "Test Author");
        assert_eq!(email, "test@example.com");

        Ok(())
    }
}
