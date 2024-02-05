use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::prelude::*;

pub fn compress_content(content: &str) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content.as_bytes()).unwrap();
    encoder.finish().unwrap()
}
