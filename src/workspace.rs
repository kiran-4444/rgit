use anyhow::{anyhow, Result};
use walkdir::WalkDir;

use std::collections::BTreeMap;
use std::fs::{self};
use std::path::PathBuf;

use crate::index::Stat;
use crate::utils::get_root_path;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub stat: Stat,
    pub oid: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    pub name: String,
    pub path: PathBuf,
    pub children: BTreeMap<String, FileOrDir>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileOrDir {
    File(File),
    Dir(Dir),
}

impl FileOrDir {
    /// Get the parent directories of a file or directory.
    /// # Example:
    /// ```rust
    /// use std::path::PathBuf;
    /// use r_git::workspace::FileOrDir;
    /// let path = PathBuf::from("foo/bar/baz");
    /// let parents = FileOrDir::components(&path).unwrap();
    /// assert_eq!(parents, vec!["foo", "bar", "baz"]);
    pub fn components(path: &PathBuf) -> Result<Vec<String>> {
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
        for part in components.iter().take(components.len()) {
            parents.push(part.clone());
        }
        Ok(parents)
    }

    /// Get the parent directories of a file or directory.
    /// # Example:
    /// ```rust
    /// use std::path::PathBuf;
    /// use r_git::workspace::FileOrDir;
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

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceTree {
    pub workspace: BTreeMap<String, FileOrDir>,
}

impl WorkspaceTree {
    pub fn build(
        entry: FileOrDir,
        parents: Vec<String>,
        components: Vec<String>,
        workspace: &mut BTreeMap<String, FileOrDir>,
        oid: Option<String>,
    ) {
        if parents.len() == 1 {
            let file = FileOrDir::File(File {
                name: components[0].clone(),
                path: PathBuf::from(parents[0].clone()),
                stat: Stat::new(&PathBuf::from(parents[0].clone())),
                oid,
            });
            workspace.insert(parents[0].clone(), file);
            return;
        }

        let mut parents = parents;
        let parent = parents.remove(0);
        let mut components = components;
        let component = components.remove(0);
        let dir = FileOrDir::Dir(Dir {
            name: component.clone(),
            path: PathBuf::from(parent.clone()),
            children: BTreeMap::new(),
        });

        if !workspace.contains_key(&component) {
            workspace.insert(component.clone(), dir);
        }

        let parent_dir = workspace.get_mut(&component).unwrap();
        match parent_dir {
            FileOrDir::Dir(dir) => {
                WorkspaceTree::build(entry, parents, components, &mut dir.children, oid);
            }
            _ => (),
        }
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
        let mut files = WalkDir::new(path)
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
                        .expect("failed to strip prefix")
                        .to_path_buf(),
                    stat: Stat::new(&entry.into_path()),
                    oid: None,
                })
            })
            .collect::<Vec<File>>();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        files
    }
    pub fn new(root: Option<&PathBuf>) -> Self {
        match root {
            Some(root) => {
                let files = WorkspaceTree::list_files(root);
                let mut workspace = BTreeMap::new();
                for file in files {
                    let parents = FileOrDir::parent_directories(&file.path)
                        .expect("failed to get parent directories");
                    let path_components =
                        FileOrDir::components(&file.path).expect("failed to get parent components");
                    if parents.len() > 1 {
                        let dir_entry = FileOrDir::Dir(Dir {
                            name: path_components[0].clone(),
                            path: PathBuf::from(parents[0].clone()),
                            children: BTreeMap::new(),
                        });
                        WorkspaceTree::build(
                            dir_entry,
                            parents,
                            path_components,
                            &mut workspace,
                            None,
                        );
                    } else {
                        let file_entry = FileOrDir::File(File {
                            name: file.name.clone(),
                            path: file.path.clone(),
                            stat: file.stat.clone(),
                            oid: None,
                        });
                        workspace.insert(
                            file.path.as_os_str().to_str().unwrap().to_owned(),
                            file_entry,
                        );
                    }
                }
                WorkspaceTree { workspace }
            }
            None => WorkspaceTree {
                workspace: BTreeMap::new(),
            },
        }
    }
}
