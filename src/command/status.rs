use clap::Parser;
use std::path::PathBuf;

use crate::workspace;

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) {
        let workspace = workspace::Workspace::new(PathBuf::from("."));
        let files = workspace.list_files().unwrap();
        for file in files {
            println!("{}", file.0.unwrap());
        }
    }
}
