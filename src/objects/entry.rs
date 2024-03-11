#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Entry {
    pub name: String,
    pub oid: String,
    pub mode: String,
}

impl Entry {
    pub fn new(name: String, oid: String, mode: String) -> Self {
        Self { name, oid, mode }
    }
}
