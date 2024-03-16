use clap::Parser;

use crate::objects::{Blob, Index};
use crate::{database, workspace};

#[derive(Parser, Debug, PartialEq)]
pub struct AddCMD {
    files: Vec<String>,
}

impl AddCMD {
    pub fn run(&self) {
        let current_dir = std::env::current_dir().expect("failed to get current directory");
        let git_path = current_dir.join(".rgit");

        let workspace = workspace::Workspace::new(current_dir);
        let database = database::Database::new(git_path.join("objects"));
        let mut index = Index::new(git_path.join("index"));

        for file in &self.files {
            self.add_file(&file, &workspace, &database, &mut index);
        }
    }

    fn add_file(
        &self,
        file: &str,
        workspace: &workspace::Workspace,
        database: &database::Database,
        index: &mut Index,
    ) {
        let files = workspace.list_files(
            std::env::current_dir()
                .expect("failed to get current dir")
                .join(file),
        );

        files.iter().for_each(|entry| {
            let stat = workspace.get_file_stat(&entry.name);
            let data = workspace.read_file(&entry.name);
            let mut blob = Blob::new(data);
            database.store(&mut blob);
            let oid = blob.oid.expect("failed to get oid");
            index.add(&entry.name, oid, stat);
        });
        index.write_updates();
    }
}
