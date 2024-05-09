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

    pub fn parse(oid: String, content: Vec<u8>) -> Self {
        let content = String::from_utf8(content).unwrap();
        Self {
            oid: Some(oid),
            data: content,
        }
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
