use clap::Parser;

use crate::utils::list_files;

const IGNORE: [&str; 7] = [
    ".",
    "..",
    ".rgit",
    ".git",
    ".pgit",
    "pgit.py",
    ".mypy_cache",
];

#[derive(Parser, Debug, PartialEq)]
pub struct StatusCMD {}

impl StatusCMD {
    pub fn run(self) {
        let files = list_files().unwrap();
        for file in files {
            println!("{}", file);
        }
    }
}
