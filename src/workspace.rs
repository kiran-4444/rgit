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
        let is_executable = file_path
            .metadata()
            .expect("failed to read file metadata")
            .permissions()
            .mode()
            & 0o111
            != 0;
        // git uses 100644 for normal files and 100755 for executable files
        let file_mode = if is_executable { "100755" } else { "100644" };
        file_mode.to_string()
    }

    fn get_file_name(&self, entry: &Path) -> String {
        entry
            .file_name()
            .expect("failed to get file name")
            .to_str()
            .expect("failed to convert file name to str")
            .to_string()
    }

    fn _list_files(&self, vec: &mut Vec<PathBuf>, path: &Path) {
        if self.ignored_files().contains(&self.get_file_name(&path)) {
            return;
        }
        if metadata(&path).expect("failed to get metadata").is_dir() {
            let paths = fs::read_dir(&path).expect("failed to read dir");
            println!("{:?}", paths);
            for path_result in paths {
                let full_path = path_result.expect("failed to get path").path();
                if metadata(&full_path)
                    .expect("failed to get metadata")
                    .is_dir()
                {
                    self._list_files(vec, &full_path);
                } else {
                    vec.push(full_path);
                }
            }
        } else {
            vec.push(path.to_path_buf());
        }
    }

    pub fn read_file(&self, file_path: &PathBuf) -> String {
        fs::read_to_string(file_path).expect("failed to read file")
    }

    pub fn get_file_stat(&self, file_path: &PathBuf) -> std::fs::Metadata {
        fs::metadata(file_path).expect("failed to get file metadata")
    }

    pub fn list_files(&self, path: PathBuf) -> Vec<WorkSpaceEntry> {
        let mut vec = Vec::new();
        self._list_files(&mut vec, &path);
        let mut entries = vec
            .iter()
            .map(|path| WorkSpaceEntry {
                name: path
                    .strip_prefix(std::env::current_dir().expect("failed to get current dir"))
                    .expect("failed to strip prefix")
                    .to_owned(),
                mode: self.get_mode(&path),
            })
            .collect::<Vec<WorkSpaceEntry>>();

        entries.sort_by(|a, b| {
            a.name
                .to_owned()
                .to_str()
                .expect("failed to convert path to str")
                .to_owned()
                .cmp(
                    &b.name
                        .to_owned()
                        .to_str()
                        .expect("failed to convert path to str")
                        .to_owned(),
                )
        });
        entries
    }
}
