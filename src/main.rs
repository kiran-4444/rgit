mod command;
mod database;
mod objects;

pub mod utils;

use clap::Parser;
use command::GitCMD;
use std::env;

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
    fn run(self) {
        self.git_command.run()
    }
}

fn main() {
    RGit::parse().run()
}
