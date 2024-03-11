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
                    // print file mode
                    let file_mode = format!("{:o}", e.metadata().unwrap().permissions().mode());
                    let file_name = e.file_name().to_string_lossy().into_owned();
                    if !ignore.contains(&file_name) {
                        Some((Some(file_name), file_mode))
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(files)
    }
}
