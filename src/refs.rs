use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Refs {
    pub git_path: std::path::PathBuf,
}

impl Refs {
    pub fn new(git_path: PathBuf) -> Self {
        Refs { git_path }
    }

    pub fn update_head(&self, oid: &str) {
        let head = self.git_path.join("HEAD");
        std::fs::write(head, oid).unwrap();
    }

    pub fn head_path(&self) -> std::path::PathBuf {
        self.git_path.join("HEAD")
    }

    pub fn read_head(&self) -> Option<String> {
        match self.head_path().exists() {
            true => Some(std::fs::read_to_string(self.head_path()).unwrap()),
            false => None,
        }
    }
}
