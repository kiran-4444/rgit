use anyhow::{anyhow, Result};
use walkdir::WalkDir;

use std::collections::BTreeMap;
use std::fs::{self, Metadata};
use std::path::PathBuf;

use crate::utils::get_root_path;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Dir {
    pub name: String,
    pub children: BTreeMap<String, FileOrDir>,
}

#[derive(Debug, Clone)]
pub enum FileOrDir {
    File(File),
    Dir(Dir),
}

impl FileOrDir {
    /// Get the parent directories of a file or directory.
    /// # Example:
    /// ```rust
    /// use std::path::PathBuf;
    /// use r_git::workspace_tree::FileOrDir;
    /// let path = PathBuf::from("foo/bar/baz");
    /// let parents = FileOrDir::parent_directories(&path).unwrap();
    /// assert_eq!(parents, vec!["foo", "foo/bar", "foo/bar/baz"]);
    pub fn parent_directories(path: &PathBuf) -> Result<Vec<String>> {
        let mut parents = Vec::new();
        let components = path
            .components()
            .map(|c| {
                Ok(c.as_os_str()
                    .to_str()
                    .expect("failed to convert path to str")
                    .to_owned())
            })
            .collect::<Result<Vec<_>>>()?;
        let mut current_path = String::new();
        for part in components.iter().take(components.len()) {
            current_path.push_str(part);
            parents.push(current_path.clone());
            current_path.push('/');
        }
        Ok(parents)
    }
}

#[derive(Debug)]
pub struct WorkspaceTree {
    pub workspace: BTreeMap<String, FileOrDir>,
}

impl WorkspaceTree {
    pub fn build(
        entry: FileOrDir,
        parents: Vec<String>,
        workspace: &mut BTreeMap<String, FileOrDir>,
    ) {
        if parents.len() == 1 {
            let file = FileOrDir::File(File {
                name: parents[0].clone(),
                path: PathBuf::from(parents[0].clone()),
            });
            workspace.insert(parents[0].clone(), file);
            return;
        }

        let parent = parents[0].clone();
        let mut parents = parents;
        parents.remove(0);
        let dir = FileOrDir::Dir(Dir {
            name: parent.clone(),
            children: BTreeMap::new(),
        });

        if !workspace.contains_key(&parent) {
            workspace.insert(parent.clone(), dir);
        }

        let parent_dir = workspace.get_mut(&parent).unwrap();
        match parent_dir {
            FileOrDir::Dir(dir) => {
                WorkspaceTree::build(entry, parents, &mut dir.children);
            }
            _ => (),
        }
    }

    pub fn get_file_stat(&self, file_path: &PathBuf) -> Result<Metadata> {
        Ok(fs::metadata(file_path)?)
    }

    pub fn read_file(&self, file_path: &PathBuf) -> Result<Vec<u8>> {
        let content = fs::read(file_path)
            .map_err(|e| anyhow!("failed to read file {}: {}", file_path.display(), e))?;
        Ok(content)
    }

    fn ignored_files() -> Result<Vec<String>> {
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

    pub fn list_files(path: &PathBuf) -> Vec<File> {
        let ignored_files = WorkspaceTree::ignored_files().expect("failed to get ignored files");
        dbg!(&ignored_files);
        WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let path = entry.path();
                let components = path.components().map(|c| c.as_os_str()).collect::<Vec<_>>();
                let mut is_ignored = false;
                for ignored in ignored_files.iter() {
                    if components.contains(&ignored.as_ref()) {
                        is_ignored = true;
                        break;
                    }
                }
                !is_ignored
            })
            .filter_map(|entry| {
                entry.file_type().is_file().then(|| File {
                    name: entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    path: entry
                        .path()
                        .strip_prefix(&std::env::current_dir().unwrap())
                        .unwrap()
                        .to_path_buf(),
                })
            })
            .collect::<Vec<File>>()
    }
    pub fn new(root: &PathBuf) -> Self {
        let files = WorkspaceTree::list_files(root);
        let mut workspace = BTreeMap::new();
        for file in files {
            let parents = FileOrDir::parent_directories(&file.path)
                .expect("failed to get parent directories");
            if parents.len() > 1 {
                let dir_entry = FileOrDir::Dir(Dir {
                    name: parents[0].clone(),
                    children: BTreeMap::new(),
                });
                WorkspaceTree::build(dir_entry, parents, &mut workspace);
            } else {
                let file_entry = FileOrDir::File(File {
                    name: file.name.clone(),
                    path: file.path.clone(),
                });
                workspace.insert(file.name.clone(), file_entry);
            }
        }
        WorkspaceTree { workspace }
    }
}
