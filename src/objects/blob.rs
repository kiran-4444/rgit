use super::storable::Storable;

#[derive(Debug, Clone)]
pub struct Blob {
    pub data: String,
    pub oid: Option<String>,
}

impl Blob {
    pub fn new(data: &str) -> Self {
        Self {
            oid: None,
            data: data.to_owned(),
        }
    }
}

impl Storable for Blob {
    fn set_oid(&mut self, oid: &str) {
        self.oid = Some(oid.to_owned());
    }

    fn blob_type(&self) -> &str {
        "blob"
    }

    fn data(&self) -> String {
        self.data.to_owned()
    }
}
