use std::fs;

const IGNORE: [&str; 7] = [
    ".",
    "..",
    ".rgit",
    ".git",
    ".pgit",
    "pgit.py",
    ".mypy_cache",
];

pub struct Workspace {
    pathname: String,
}

impl Workspace {
    pub fn new(pathname: &str) -> Self {
        Self {
            pathname: pathname.to_string(),
        }
    }

    pub fn read_file(&self, file: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(file)
    }

    pub fn list_files(&self) -> Result<Vec<String>, std::io::Error> {
        let entries = fs::read_dir(&self.pathname)?;

        // iterate through the files in the current directory by skipping the IGNORE files or directories
        let files: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let file_name = e.file_name().to_string_lossy().into_owned();
                    if !IGNORE.contains(&file_name.as_str()) {
                        Some(file_name)
                    } else {
                        None
                    }
                })
            })
            .collect();
        Ok(files)
    }
}
