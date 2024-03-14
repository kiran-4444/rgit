use std::fs::OpenOptions;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Lockfile {
    pub file_path: PathBuf,
    pub lock_file_path: Option<PathBuf>,
    pub lock: Option<std::fs::File>,
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

    pub fn hold_for_update(&mut self) -> bool {
        if self.lock.is_none() {
            // If the lock file already exists, we return back false
            if self
                .lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref")
                .exists()
            {
                println!("Lock file already exists");
                return false;
            }

            // If the lock file does not exist, we create it and hold the lock.
            // If the lock file already exists, we need to error out
            self.lock = Some(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .create_new(true)
                    .open(self.lock_file_path.as_ref().unwrap())
                    .expect("Failed to open lock file"),
            );
            true
        } else {
            if !self
                .lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref")
                .exists()
            {
                panic!("Lockfile is missing, but lock is held");
            }

            // If the lock file is still there, we still hold the lock
            false
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        if self.lock.is_none() {
            panic!("Lock is not held");
        }
        std::io::Write::write_all(
            &mut self.lock.as_ref().expect("failed to get lock ref"),
            data,
        )
        .unwrap();
    }

    pub fn commit(&mut self) {
        if self.lock.is_none() {
            panic!("Lock is not held");
        }

        // We rename the lock file to the file path
        std::fs::rename(
            self.lock_file_path
                .as_ref()
                .expect("failed to get lock_file_path ref"),
            self.file_path.as_path(),
        )
        .expect("failed to rename lock file to file path");

        // We release the lock
        self.lock = None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn update_head_should_succeed_if_lock_can_be_held() {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        assert_eq!(lockfile.hold_for_update(), true);
        lockfile.write(b"test data");
        lockfile.commit();

        assert!(PathBuf::from("HEAD").exists());
        assert!(PathBuf::from("HEAD.lock").exists() == false);

        // clean up
        std::fs::remove_file("HEAD").expect("failed to remove HEAD");
    }

    #[test]
    fn update_head_should_fail_if_lock_cannot_be_held() {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));

        // Create a lock file
        let _lock_file = File::create("HEAD.lock").expect("failed to create lock file");

        assert_eq!(lockfile.hold_for_update(), false);

        // Clean up
        // this may not delete the file immediately, but it will be deleted eventually
        std::fs::remove_file("HEAD.lock").expect("failed to remove HEAD.lock");
    }

    #[test]
    fn write_should_fail_if_lock_is_not_held() {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        let data = b"test data";

        assert_eq!(lockfile.hold_for_update(), true);
        lockfile.write(data);
        assert_eq!(
            std::fs::read_to_string("HEAD.lock").expect("failed to read lock file"),
            "test data"
        );

        // lock is still held, so this should fail
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        assert!(PathBuf::from("HEAD.lock").exists() == true);
        assert_eq!(lockfile.hold_for_update(), false);

        // clean up
        std::fs::remove_file("HEAD.lock").expect("failed to remove HEAD.lock");
    }

    #[test]
    fn write_and_commit_should_pass() {
        let mut lockfile = Lockfile::new(PathBuf::from("HEAD"));
        let data = b"test data";

        assert_eq!(lockfile.hold_for_update(), true);
        lockfile.write(data);
        lockfile.commit();

        assert_eq!(
            std::fs::read_to_string("HEAD").expect("failed to read lock file"),
            "test data"
        );
        assert!(PathBuf::from("HEAD.lock").exists() == false);

        // clean up
        std::fs::remove_file("HEAD").expect("failed to remove HEAD");
    }
}
