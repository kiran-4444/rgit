pub trait Storable {
    fn set_oid(&mut self, oid: &str);
    fn blob_type(&self) -> &str;
    fn data(&self) -> String;
}
