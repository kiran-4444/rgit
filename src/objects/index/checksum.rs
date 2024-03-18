use anyhow::Result;
use sha1::{Digest, Sha1};
use std::io::{prelude::*, BufReader};

static CHECKSUM_SIZE: usize = 20;

#[derive(Debug)]
pub struct Checksum {
    data: Vec<u8>,
    hasher: Sha1,
}

impl Checksum {
    pub fn new(data: &[u8]) -> Self {
        let hasher = Sha1::new();
        Self {
            data: data.to_vec(),
            hasher,
        }
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut reader = BufReader::new(&self.data[..]);
        let mut buffer = vec![0; size];
        reader.read_exact(&mut buffer[..size])?;
        self.hasher.update(&buffer[..size]);
        Ok(buffer)
    }

    pub fn verify_checksum(&mut self) -> Result<()> {
        let checksum = self.read(CHECKSUM_SIZE)?;
        let expected: Vec<u8> = self.hasher.clone().finalize().to_vec();
        if checksum != expected {
            anyhow::bail!("Checksum mismatch");
        }
        Ok(())
    }
}
