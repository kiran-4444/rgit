use crate::objects::Author;

use super::storable::Storable;

#[derive(Debug, Clone)]
pub struct Commit {
    pub oid: Option<String>,
    pub tree: String,
    pub author: Author,
    pub message: String,
}

impl Commit {
    pub fn new(tree: &str, author: Author, message: &str) -> Self {
        Self {
            oid: None,
            tree: tree.to_string(),
            author,
            message: message.to_string(),
        }
    }
}

impl Storable for Commit {
    fn set_oid(&mut self, oid: &str) {
        self.oid = Some(oid.to_string());
    }

    fn blob_type(&self) -> &str {
        "commit"
    }

    fn data(&self) -> String {
        format!(
            "tree {}\nauthor {}\ncomitter {}\n\n{}\n",
            self.tree, self.author, self.author, self.message
        )
    }
}
