use std::{env, path::Path};

use clap::{arg, Parser};
use std::io::Write;

use crate::{command::status::StatusCMD, database, objects};

use super::init::construct_git_path;

#[derive(Parser, Debug, PartialEq)]
pub struct CommitCMD {
    #[arg(short)]
    message: String,
}

impl CommitCMD {
    pub fn run(&self) {
        let git_path = construct_git_path(&Path::new("."));
        let db_path = git_path.join("objects");
        let db = database::Database::new(db_path.to_str().unwrap());
        let status = StatusCMD {};
        let files = status.list_files().unwrap();
        let entries = files
            .iter()
            .map(|file| {
                // println!("File: {}", file);
                // let data = workspace.read_file(file).expect("Error reading file");
                let data = std::fs::read_to_string(file).unwrap();
                let mut blob = objects::Blob::new(&data);
                db.store(&mut blob);
                objects::Entry::new(&file, &blob.oid.unwrap())
            })
            .collect::<Vec<objects::Entry>>();

        let mut tree = objects::Tree::new(entries);
        db.store(&mut tree);
        println!("{:?}", tree.oid);

        let name = env::var("RGIT_AUTHOR_NAME").unwrap();
        let email = env::var("RGIT_AUTHOR_EMAIL").unwrap();
        let author = objects::Author::new(&name, &email);
        println!("{:?}", author);
        let message = "Initial commit";

        let mut commit = objects::Commit::new(&tree.oid.unwrap(), author, message);
        db.store(&mut commit);
        println!("{:?}", commit);

        let head = git_path.join("HEAD");
        let mut file = std::fs::File::create(head).unwrap();
        file.write(commit.oid.to_owned().unwrap().as_bytes())
            .unwrap();

        println!("[(root-commit) {}] {}", commit.oid.unwrap(), message);
    }
}
