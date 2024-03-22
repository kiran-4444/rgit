use anyhow::Result;
use clap::Parser;
use command::GitCMD;
use std::env;

mod command;
mod database;
mod index;
mod lockfile;
mod refs;
mod utils;
mod workspace;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

/// A simple git clone written in Rust
#[derive(Debug, Parser, PartialEq)]
#[command(author = "Chandra Kiran G", version = VERSION, help_template(HELP_TEMPLATE))]
struct RGit {
    #[command(subcommand)]
    git_command: GitCMD,
}

impl RGit {
    fn run(self) -> Result<()> {
        self.git_command.run()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    RGit::parse().run()?;
    Ok(())
}
