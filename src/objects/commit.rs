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
    pub fn new(tree: String, author: Author, message: String) -> Self {
        Self {
            oid: None,
            tree,
            author,
            message,
        }
    }
}

impl Storable for Commit {
    fn set_oid(&mut self, oid: String) {
        self.oid = Some(oid);
    }

    fn blob_type(&self) -> String {
        "commit".to_owned()
    }

    fn data(&self) -> String {
        format!(
            "tree {}\nauthor {}\ncomitter {}\n\n{}\n",
            self.tree, self.author, self.author, self.message
        )
    }
}
