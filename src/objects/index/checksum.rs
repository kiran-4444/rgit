use anyhow::Result;
use sha1::{Digest, Sha1};
use std::{
    fs::File,
    io::{prelude::*, BufReader, SeekFrom},
};

static CHECKSUM_SIZE: usize = 20;

#[derive(Debug)]
pub struct Checksum<'a> {
    file: &'a mut File,
    hasher: Sha1,
}

impl<'a> Checksum<'a> {
    pub fn new(file: &'a mut File) -> Self {
        let hasher = Sha1::new();
        Self { file, hasher }
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut file = self.file.try_clone()?;
        let before = file.stream_position()?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; size];
        reader.read_exact(&mut buffer)?;
        self.hasher.update(&buffer);
        reader.seek(SeekFrom::Start(before + size as u64))?;
        Ok(buffer)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        println!("Writing data: {:?}", data);
        self.file.write_all(data)?;
        self.hasher.update(data);
        Ok(())
    }

    pub fn write_checksum(&mut self) -> Result<()> {
        let checksum = self.hasher.clone().finalize().to_vec();
        println!("Writing checksum: {:?}", checksum);
        self.write(&checksum)?;
        Ok(())
    }

    pub fn verify_checksum(&mut self) -> Result<()> {
        println!("Verifying checksum");
        let file = self.file.try_clone()?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; 20];
        reader.read_exact(&mut buffer)?;
        let checksum = buffer;
        let expected: Vec<u8> = self.hasher.clone().finalize().to_vec();
        println!("Checksum: {:?}, Expected: {:?}", checksum, expected);
        if checksum != expected {
            anyhow::bail!("Checksum mismatch");
        }
        Ok(())
    }
}
