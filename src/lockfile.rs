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
            if self.lock_file_path.as_ref().unwrap().exists() {
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
                    .unwrap(),
            );
            true
        } else {
            if !self.lock_file_path.as_ref().unwrap().exists() {
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
        std::io::Write::write_all(&mut self.lock.as_ref().unwrap(), data).unwrap();
    }

    pub fn commit(&mut self) {
        if self.lock.is_none() {
            panic!("Lock is not held");
        }

        // We rename the lock file to the file path
        std::fs::rename(
            self.lock_file_path.as_ref().unwrap(),
            self.file_path.as_path(),
        )
        .unwrap();

        // We release the lock
        self.lock = None;
    }
}
