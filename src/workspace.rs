use std::fs::{self, metadata};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, os::unix::fs::PermissionsExt};

#[derive(Debug)]
pub struct WorkSpaceEntry {
    pub name: PathBuf,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: std::path::PathBuf,
}

impl Workspace {
    pub fn new(root_path: std::path::PathBuf) -> Self {
        Workspace { root_path }
    }

    fn ignored_files(&self) -> Vec<String> {
        let ignores = vec![
            ".",
            "..",
            ".rgit",
            ".git",
            ".pgit",
            "pgit.py",
            ".mypy_cache",
        ];

        ignores.iter().map(|s| s.to_string()).collect()
    }

    fn get_mode(&self, file_path: &PathBuf) -> String {
        let is_executable = file_path.metadata().unwrap().permissions().mode() & 0o111 != 0;
        // git uses 100644 for normal files and 100755 for executable files
        let file_mode = if is_executable { "100755" } else { "100644" };
        file_mode.to_string()
    }

    fn get_file_name(&self, entry: &Path) -> String {
        entry.file_name().unwrap().to_str().unwrap().to_string()
    }

    fn _list_files(&self, vec: &mut Vec<PathBuf>, path: &Path) {
        if self.ignored_files().contains(&self.get_file_name(&path)) {
            return;
        }
        if metadata(&path).unwrap().is_dir() {
            let paths = fs::read_dir(&path).unwrap();
            for path_result in paths {
                let full_path = path_result.unwrap().path();
                if metadata(&full_path).unwrap().is_dir() {
                    self._list_files(vec, &full_path);
                } else {
                    vec.push(full_path);
                }
            }
        }
    }

    pub fn list_files(&self, dir_path: PathBuf) -> Vec<WorkSpaceEntry> {
        let mut vec = Vec::new();
        self._list_files(&mut vec, &dir_path);
        vec.iter()
            .map(|path| WorkSpaceEntry {
                name: path
                    .strip_prefix(std::env::current_dir().unwrap())
                    .unwrap()
                    .to_owned(),

                mode: self.get_mode(&path),
            })
            .collect()
    }
}
