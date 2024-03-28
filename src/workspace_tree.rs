use anyhow::Result;
use predicates::path;
use std::collections::BTreeMap;
use std::fs::Metadata;
use std::path::PathBuf;

use crate::workspace::{self, WorkSpaceEntry};

#[derive(Debug, Clone)]
pub enum FileOrDir {
    File(File),
    Dir(Dir),
}

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    // pub stat: Metadata,
}

impl FileOrDir {
    pub fn parent_directories(&self) -> Result<Vec<String>> {
        let path = match self {
            FileOrDir::File(file) => &file.path,
            FileOrDir::Dir(dir) => &dir.path,
        };
        let components = PathBuf::from(path)
            .components()
            .map(|c| {
                Ok(c.as_os_str()
                    .to_str()
                    .expect("failed to convert path to str")
                    .to_owned())
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(components)
    }
}

#[derive(Debug, Clone)]
pub struct Dir {
    pub path: PathBuf,
    pub children: BTreeMap<String, FileOrDir>,
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
        if parents.len() > 1 {
            dbg!(parents.clone());
            let mut child = BTreeMap::new();
            WorkspaceTree::build(entry.clone(), parents[1..].to_vec(), &mut child);
            match entry {
                FileOrDir::File(file) => {
                    let file_or_dir = FileOrDir::Dir(Dir {
                        path: file.path.to_owned(),
                        children: child.clone(),
                    });
                    let check_key = workspace.get_mut(&parents[0]);

                    match check_key {
                        Some(FileOrDir::Dir(dir)) => {
                            dir.children.insert(parents[0].clone(), file_or_dir);
                        }
                        _ => {
                            let dir = FileOrDir::Dir(Dir {
                                path: PathBuf::from(&parents[0]),
                                children: child.clone(),
                            });
                            workspace.insert(parents[0].clone(), dir);
                        }
                    }
                }
                _ => panic!("Invalid entry type"),
            }
        } else {
            match entry {
                FileOrDir::File(file) => {
                    let file_or_dir = FileOrDir::File(File {
                        path: file.path.to_owned(),
                        // stat: file.stat,
                    });
                    workspace.insert(file.path.to_str().unwrap().to_owned(), file_or_dir);
                }
                _ => panic!("Invalid entry type"),
            }
        }
    }

    pub fn new(root: Vec<WorkSpaceEntry>) -> Self {
        let mut workspace = BTreeMap::new();
        for entry in root {
            let entry = FileOrDir::File(File {
                path: entry.name.clone(),
                // stat: entry.name.metadata().unwrap(),
            });
            let parents = entry.parent_directories().unwrap();
            WorkspaceTree::build(entry, parents, &mut workspace);
        }
        WorkspaceTree { workspace }
    }
}
