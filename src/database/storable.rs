pub trait Storable {
    fn set_oid(&mut self, oid: String);
    fn blob_type(&self) -> String;
    fn data(&self) -> String;
    // fn parse(oid: String, content: Vec<u8>) -> Self;
}
