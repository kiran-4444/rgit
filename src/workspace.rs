use anyhow::{anyhow, Result};
use std::fs::{self, metadata, Metadata};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, os::unix::fs::PermissionsExt};

use crate::utils::get_root_path;
#[derive(Debug)]
pub struct WorkSpaceEntry {
    pub name: PathBuf,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: PathBuf,
}

impl Workspace {
    pub fn new(root_path: PathBuf) -> Self {
        Workspace { root_path }
    }

    fn ignored_files(&self) -> Result<Vec<String>> {
        let root_path = get_root_path()?;
        let gitignore_path = root_path.join(".rgitignore");
        let mut ignored_files = vec![];
        if gitignore_path.exists() {
            let content = fs::read_to_string(gitignore_path).expect("failed to read .rgitignore");
            let content = content.trim();
            ignored_files = content
                .split("\n")
                .map(|s| s.to_string().trim_matches('/').to_string())
                .collect::<Vec<String>>();
        }
        ignored_files.push(".rgit".to_string());
        Ok(ignored_files)
    }

    fn get_mode(&self, file_path: &PathBuf) -> Result<String> {
        let is_executable = file_path.metadata()?.permissions().mode() & 0o111 != 0;
        // git uses 100644 for normal files and 100755 for executable files
        let file_mode = if is_executable { "100755" } else { "100644" };
        Ok(file_mode.to_string())
    }

    fn get_file_name(&self, entry: &Path) -> Result<String> {
        Ok(entry
            .file_name()
            .ok_or(anyhow!("failed to get file name"))?
            .to_str()
            .ok_or(anyhow!("failed to convert path to str"))?
            .to_string())
    }

    fn _list_files(&self, vec: &mut Vec<PathBuf>, path: &Path) -> Result<()> {
        if metadata(&path)?.is_dir() {
            let last_component = path.components().last().unwrap();
            if self
                .ignored_files()?
                .contains(&last_component.as_os_str().to_str().unwrap().to_string())
            {
                return Ok(());
            }
            let paths = fs::read_dir(&path)?;
            for path_result in paths {
                let full_path = path_result?.path();
                if metadata(&full_path)?.is_dir() {
                    self._list_files(vec, &full_path)?;
                } else {
                    let file_name = self.get_file_name(&full_path)?;
                    if self.ignored_files()?.contains(&file_name) {
                        continue;
                    }
                    vec.push(full_path);
                }
            }
            Ok(())
        } else {
            vec.push(path.to_path_buf());
            Ok(())
        }
    }

    pub fn read_file(&self, file_path: &PathBuf) -> Result<Vec<u8>> {
        let content = fs::read(file_path)
            .map_err(|e| anyhow!("failed to read file {}: {}", file_path.display(), e))?;
        Ok(content)
    }

    pub fn get_file_stat(&self, file_path: &PathBuf) -> Result<Metadata> {
        Ok(fs::metadata(file_path)?)
    }

    pub fn list_files(&self, path: &PathBuf) -> Result<Vec<WorkSpaceEntry>> {
        let mut vec = Vec::new();
        self._list_files(&mut vec, &path)?;
        let mut entries = vec
            .iter()
            .map(|path| {
                Ok(WorkSpaceEntry {
                    name: path.strip_prefix(std::env::current_dir()?)?.to_owned(),
                    mode: self.get_mode(&path)?,
                })
            })
            .collect::<Result<Vec<WorkSpaceEntry>>>()?;

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
        Ok(entries)
    }
}
