use crate::objects::Author;

use super::storable::Storable;

#[derive(Debug, Clone)]
pub struct Commit<'a> {
    pub oid: Option<String>,
    pub tree: String,
    pub author: Author<'a>,
    pub message: &'a str,
}

impl<'a> Commit<'a> {
    pub fn new(tree: String, author: Author<'a>, message: &'a str) -> Self {
        Self {
            oid: None,
            tree,
            author,
            message,
        }
    }
}

impl<'a> Storable for Commit<'a> {
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
