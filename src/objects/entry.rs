use std::path::Path;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Hash, Ord)]
pub struct Entry {
    pub name: String,
    pub oid: String,
    pub mode: String,
}

impl Entry {
    pub fn new(name: String, oid: String, mode: String) -> Self {
        Self { name, oid, mode }
    }

    pub fn parent_directories(&self) -> Vec<String> {
        let components = Path::new(&self.name)
            .components()
            .map(|c| c.as_os_str().to_str().expect("Invalid path"))
            .collect::<Vec<_>>();
        let mut parents = Vec::new();
        let mut current_path = String::new();
        for part in components.iter().take(components.len() - 1) {
            current_path.push_str(part);
            parents.push(current_path.clone());
            current_path.push('/');
        }
        parents
    }
}
