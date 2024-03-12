use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Entry {
    pub name: String,
    pub oid: String,
    pub mode: String,
    pub path: PathBuf,
}

impl Entry {
    pub fn new(name: String, oid: String, mode: String, path: PathBuf) -> Self {
        Self {
            name,
            oid,
            mode,
            path,
        }
    }

    pub fn parent_directories(&self) -> Vec<String> {
        let mut parents = Path::new(&self.name)
            .parent()
            .unwrap()
            .iter()
            .map(|x| x.to_str().unwrap().to_string())
            .collect::<Vec<String>>();

        parents.reverse();
        parents
    }
}
