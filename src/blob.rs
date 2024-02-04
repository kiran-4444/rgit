pub struct Blob {
    pub data: String,
    pub oid: String,
}

impl Blob {
    pub fn new(data: &str) -> Self {
        Self {
            oid: data.to_owned(),
            data: data.to_owned(),
        }
    }

    pub fn blob_type(&self) -> &str {
        "blob"
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}
