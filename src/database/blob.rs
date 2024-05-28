use std::path::PathBuf;

use crate::database::{storable::Storable, Content};

#[derive(Debug, Clone)]
pub struct Blob {
    pub data: String,
    pub oid: Option<String>,
}

impl Blob {
    pub fn new(data: String) -> Self {
        Self { oid: None, data }
    }

    pub fn parse(oid: String) -> Self {
        let object_store = PathBuf::from(".rgit/objects");
        let content = String::from_utf8(
            Content::parse(&oid, object_store)
                .expect("Failed to parse content")
                .body,
        )
        .expect("Failed to convert content to string");
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
