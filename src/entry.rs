#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub oid: String,
}

impl Entry {
    pub fn new(name: &str, oid: &str) -> Self {
        Self {
            name: name.to_owned(),
            oid: oid.to_owned(),
        }
    }
}
