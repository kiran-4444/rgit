use clap::Subcommand;

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
}

impl GitCMD {
    pub fn run(self) {
        match self {
            GitCMD::Init(init) => init.run(),
            GitCMD::Commit(commit) => commit.run(),
            GitCMD::Status(status) => status.run(),
        }
    }
}
