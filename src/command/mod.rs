use anyhow::Result;
use clap::Subcommand;

mod add;
mod branch;
mod commit;
mod diff;
mod init;
mod log;
mod status;

#[derive(Subcommand, Debug)]
pub enum GitCMD {
    /// Initialize a new git repository
    Init(init::InitCMD),

    /// Commit the changes in the working tree
    Commit(commit::CommitCMD),

    /// Show the working tree status
    Status(status::StatusCMD),

    /// Add file contents to the index
    Add(add::AddCMD),

    /// Show diff
    Diff(diff::DiffCMD),

    /// Branch operations
    Branch(branch::BranchCMD),

    /// Log
    Log(log::LogCMD),
}

impl GitCMD {
    pub fn run(self) -> Result<()> {
        match self {
            GitCMD::Init(init) => init.run()?,
            GitCMD::Commit(commit) => commit.run()?,
            GitCMD::Status(status) => status.run()?,
            GitCMD::Add(add) => add.run()?,
            GitCMD::Diff(diff) => diff.run()?,
            GitCMD::Branch(branch) => branch.run()?,
            GitCMD::Log(log) => log.run()?,
        }
        Ok(())
    }
}
