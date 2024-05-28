use crate::{
    database::{Commit, Database, ParsedContent},
    refs::{parse_revision, Refs, Revision},
    utils::{get_root_path, write_to_stdout},
};
use anyhow::Result;
use clap::Parser;

use super::commit;

#[derive(Debug, Parser, PartialEq, Eq)]
pub struct LogCMD {}

impl LogCMD {
    pub fn run(self) -> Result<()> {
        let root_path = get_root_path()?;
        let git_path = root_path.join(".rgit");
        let refs = Refs::new(git_path.clone());
        let database = Database::new(git_path.clone().join("objects"));

        let mut head = refs.read_head().unwrap();

        let commits = refs.get_all_commits()?;

        for commit in commits.iter().rev() {
            let oid = commit.oid.clone().unwrap();
            write_to_stdout(&oid)?;
            write_to_stdout(&commit.message)?;
            write_to_stdout(&commit.author.to_string())?;
            write_to_stdout("\n")?;
        }
        Ok(())
    }
}
