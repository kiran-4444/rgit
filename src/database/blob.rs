use crate::database::storable::Storable;

#[derive(Debug, Clone)]
pub struct Blob {
    pub data: String,
    pub oid: Option<String>,
}

impl Blob {
    pub fn new(data: String) -> Self {
        Self { oid: None, data }
    }
}

impl Storable for Blob {
    fn set_oid(&mut self, oid: String) {
        self.oid = Some(oid);
    }

    fn blob_type(&self) -> String {
        "blob".to_owned()
    }

    fn data(&self) -> String {
        self.data.to_owned()
    }
}
