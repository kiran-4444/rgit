use anyhow::Result;
use predicates::path;
use std::fs::Metadata;
use std::path::PathBuf;
use std::{collections::BTreeMap, path::Path};

use crate::workspace::WorkSpaceEntry;

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

    pub fn new(root: Vec<WorkSpaceEntry>) -> Self {
        let mut workspace = BTreeMap::new();
        for entry in root {
            let parents = FileOrDir::parent_directories(&entry.name)
                .expect("failed to get parent directories");
            if parents.len() > 1 {
                let dir = FileOrDir::Dir(Dir {
                    name: parents[0].clone(),
                    children: BTreeMap::new(),
                });
                WorkspaceTree::build(dir, parents, &mut workspace);
            } else {
                let file = FileOrDir::File(File {
                    name: entry.name.file_name().unwrap().to_str().unwrap().to_owned(),
                    path: entry.name.clone(),
                });
                workspace.insert(
                    entry
                        .name
                        .file_name()
                        .expect("failed to get file name")
                        .to_str()
                        .expect("failed to convert path to str")
                        .to_owned(),
                    file,
                );
            }
        }
        WorkspaceTree { workspace }
    }
}
