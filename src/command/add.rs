use clap::Parser;

use crate::objects::{Blob, Index};
use crate::{database, workspace};

#[derive(Parser, Debug, PartialEq)]
pub struct AddCMD {
    file: String,
}

impl AddCMD {
    pub fn run(&self) {
        let current_dir = std::env::current_dir().expect("failed to get current directory");
        let git_path = current_dir.join(".rgit");

        let workspace = workspace::Workspace::new(current_dir);
        let database = database::Database::new(git_path.join("objects"));
        let mut index = Index::new(git_path.join("index"));

        let path = self.file.clone();
        let data = workspace.read_file(&path);
        let stat = workspace.get_file_stat(&path);

        let mut blob = Blob::new(data);
        database.store(&mut blob);
        let oid = blob.oid.expect("failed to get oid");
        index.add(path, oid, stat);
        index.write_updates();
    }
}
