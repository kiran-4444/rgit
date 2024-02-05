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

    pub fn set_oid(&mut self, oid: &str) {
        self.oid = Some(oid.to_owned());
    }

    pub fn blob_type(&self) -> &str {
        "blob"
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}
