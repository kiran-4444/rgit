use std::{fmt::Debug, os::unix::fs::PermissionsExt};

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

    pub fn list_files(&self) -> Result<Vec<(Option<String>, String)>, std::io::Error> {
        let entries = std::fs::read_dir(".")?;
        let ignore = self.ignored_files();
        // iterate through the files in the current directory by skipping the IGNORE files or directories
        let files: Vec<(Option<String>, String)> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let is_executable = e.metadata().unwrap().permissions().mode() & 0o111 != 0;
                    // git uses 100644 for normal files and 100755 for executable files
                    let file_mode = if is_executable { "100755" } else { "100644" };
                    let file_name = e.file_name().to_string_lossy().into_owned();
                    if !ignore.contains(&file_name) {
                        Some((Some(file_name), file_mode.to_owned()))
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(files)
    }
}
