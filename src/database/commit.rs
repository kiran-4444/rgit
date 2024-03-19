use crate::{database::storable::Storable, database::Author};

#[derive(Debug, Clone)]
pub struct Commit<'a> {
    pub parent: Option<String>,
    pub oid: Option<String>,
    pub tree: String,
    pub author: Author<'a>,
    pub message: &'a str,
}

impl<'a> Commit<'a> {
    pub fn new(parent: Option<String>, tree: String, author: Author<'a>, message: &'a str) -> Self {
        Self {
            parent,
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
        if let Some(parent) = &self.parent {
            format!(
                "tree {}\nparent {}\nauthor {}\ncomitter {}\n\n{}\n",
                self.tree, parent, self.author, self.author, self.message
            )
        } else {
            format!(
                "tree {}\nauthor {}\ncomitter {}\n\n{}\n",
                self.tree, self.author, self.author, self.message
            )
        }
    }
}
