#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Entry {
    pub name: String,
    pub oid: String,
}

impl Entry {
    pub fn new(name: String, oid: String) -> Self {
        Self { name, oid }
    }
}
