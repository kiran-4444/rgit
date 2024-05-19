use crate::{database::storable::Storable, database::Author, database::Content};

#[derive(Debug, Clone)]
pub struct Commit {
    pub parent: Option<String>,
    pub oid: Option<String>,
    pub tree: String,
    pub author: Author,
    pub message: String,
}

impl Commit {
    pub fn new(parent: Option<String>, tree: String, author: Author, message: String) -> Self {
        Self {
            parent,
            oid: None,
            tree,
            author,
            message,
        }
    }

    pub fn parse(oid: String) -> Self {
        let content = Content::parse(&oid).expect("Failed to parse content").body;

        let content = String::from_utf8(content).unwrap();
        let mut lines = content.lines();
        let tree = lines.next().unwrap().split(' ').last().unwrap();
        let parent = lines
            .next()
            .map(|line| line.split(' ').last().unwrap())
            .map(|parent| parent.to_owned());
        let author = Author::parse(lines.next().clone().unwrap());
        let message = lines.skip(2).collect::<Vec<_>>().join("\n");

        Self {
            parent,
            oid: Some(oid),
            tree: tree.to_owned(),
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
