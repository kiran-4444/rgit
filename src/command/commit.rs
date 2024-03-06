use std::{env, path::Path};

use clap::{arg, Parser};
use std::io::Write;

use crate::{database, objects, utils::list_files};

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
        let files = list_files().unwrap();
        let entries = files
            .iter()
            .map(|file| {
                let data = std::fs::read_to_string(file).unwrap();
                let mut blob = objects::Blob::new(&data);
                db.store(&mut blob);
                objects::Entry::new(&file, &blob.oid.unwrap())
            })
            .collect::<Vec<objects::Entry>>();

        let mut tree = objects::Tree::new(entries);
        db.store(&mut tree);
        println!("{:?}", tree.oid);

        let (name, email) = self.get_config();
        let author = objects::Author::new(&name, &email);
        println!("{:?}", author);

        let message = self.message.clone();
        let mut commit = objects::Commit::new(&tree.oid.unwrap(), author, message.as_str());
        db.store(&mut commit);
        println!("{:?}", commit);

        self.update_head(&commit);
        println!("[(root-commit) {}] {}", commit.oid.unwrap(), message);
    }

    fn get_config(&self) -> (String, String) {
        let name = env::var("RGIT_AUTHOR_NAME").unwrap();
        let email = env::var("RGIT_AUTHOR_EMAIL").unwrap();
        (name, email)
    }

    fn update_head(&self, commit: &objects::Commit) {
        let git_path = construct_git_path(&Path::new("."));
        let head = git_path.join("HEAD");
        let mut file = std::fs::File::create(head).unwrap();
        file.write(commit.oid.to_owned().unwrap().as_bytes())
            .unwrap();
    }
}
