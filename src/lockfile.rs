use anyhow::{bail, Result};
use cloneable_file::CloneableFile;
use std::fs::{rename, OpenOptions};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Lockfile {
    pub file_path: PathBuf,
    pub lock_file_path: Option<PathBuf>,
    pub lock: Option<CloneableFile>,
}

impl Lockfile {
    pub fn new(file_path: PathBuf) -> Self {
        let lock_file_path = file_path.with_extension("lock");
        Lockfile {
            file_path,
            lock_file_path: Some(lock_file_path),
            lock: None,
        }
    }

    pub fn rollback(&mut self) -> Result<()> {
        if self.lock.is_none() {
            bail!("Lock is not held");
        }

        // We release the lock
        self.lock = None;

        // We remove the lock file
        std::fs::remove_file(
            self.lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref"),
        )?;
        Ok(())
    }

    /// This function will create a lock file and hold the lock if the lock file does not exist.
    pub fn hold_for_update(&mut self) -> Result<bool> {
        if self.lock.is_none() {
            // If the lock file already exists, we return back false
            if self
                .lock_file_path
                .as_ref()
                .expect("failed to get reference")
                .exists()
            {
                println!("Lock file already exists");
                return Ok(false);
            }

            // If the lock file does not exist, we create it and hold the lock.
            // If the lock file already exists, we need to error out
            self.lock = Some(CloneableFile::from(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .create_new(true)
                    .open(
                        self.lock_file_path
                            .as_ref()
                            .expect("failed to get lock_file_path ref"),
                    )?,
            ));
            Ok(true)
        } else {
            if !self
                .lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref")
                .exists()
            {
                bail!("Lockfile is missing, but lock is held");
            }

            println!("Lock file already exists, but lock is held");
            // If the lock file is still there, we still hold the lock
            Ok(false)
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if self.lock.is_none() {
            bail!("Lock is not held");
        }
        std::io::Write::write_all(
            &mut self.lock.as_ref().expect("failed to get lock ref"),
            data,
        )?;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        if self.lock.is_none() {
            bail!("Lock is not held");
        }

        // We rename the lock file to the file path
        rename(
            self.lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref"),
            self.file_path.as_path(),
        )?;

        // We release the lock
        self.lock = None;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn update_head_should_succeed_if_lock_can_be_held() -> Result<()> {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        assert_eq!(lockfile.hold_for_update()?, true);
        lockfile.write(b"test data")?;
        lockfile.commit()?;

        assert!(PathBuf::from("HEAD").exists());
        assert!(PathBuf::from("HEAD.lock").exists() == false);

        // clean up
        std::fs::remove_file("HEAD")?;
        Ok(())
    }

    #[test]
    fn update_head_should_fail_if_lock_cannot_be_held() -> Result<()> {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));

        // Create a lock file
        let _lock_file = File::create("HEAD.lock")?;

        assert_eq!(lockfile.hold_for_update()?, false);

        // Clean up
        // this may not delete the file immediately, but it will be deleted eventually
        std::fs::remove_file("HEAD.lock")?;
        Ok(())
    }

    #[test]
    fn write_should_fail_if_lock_is_not_held() -> Result<()> {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        let data = b"test data";

        assert_eq!(lockfile.hold_for_update()?, true);
        lockfile.write(data)?;
        assert_eq!(std::fs::read_to_string("HEAD.lock")?, "test data");

        // lock is still held, so this should fail
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        assert!(PathBuf::from("HEAD.lock").exists() == true);
        assert_eq!(lockfile.hold_for_update()?, false);

        // clean up
        std::fs::remove_file("HEAD.lock")?;
        Ok(())
    }

    #[test]
    fn write_and_commit_should_pass() -> Result<()> {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        let data = b"test data";

        assert_eq!(lockfile.hold_for_update()?, true);
        lockfile.write(data)?;
        lockfile.commit()?;

        assert_eq!(std::fs::read_to_string("HEAD")?, "test data");
        assert!(PathBuf::from("HEAD.lock").exists() == false);

        // clean up
        std::fs::remove_file("HEAD")?;
        Ok(())
    }
}
