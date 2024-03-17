use anyhow::Result;
use clap::Subcommand;

mod add;
mod commit;
mod init;
mod status;

#[derive(Subcommand, Debug, PartialEq)]
pub enum GitCMD {
    /// Initialize a new git repository
    Init(init::InitCMD),

    /// Commit the changes in the working tree
    Commit(commit::CommitCMD),

    /// Show the working tree status
    Status(status::StatusCMD),

    /// Add file contents to the index
    Add(add::AddCMD),
}

impl GitCMD {
    pub fn run(self) -> Result<()> {
        match self {
            GitCMD::Init(init) => init.run()?,
            GitCMD::Commit(commit) => commit.run()?,
            GitCMD::Status(status) => status.run()?,
            GitCMD::Add(add) => add.run()?,
        }
        Ok(())
    }
}
