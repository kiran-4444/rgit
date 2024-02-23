mod commands;
mod database;
mod entry;
mod objects;

pub mod utils;

use clap::{Parser, Subcommand};
use commands::{construct_git_path, initialize_git_dir};
use std::env;
use std::path::Path;

use entry::Entry;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

#[derive(Subcommand, Debug, PartialEq)]
pub enum Cmd {
    #[clap(name = "init")]
    Init {
        #[clap(name = "name")]
        name: Option<String>,
    },
    #[clap(name = "commit")]
    Commit {
        #[clap(name = "message")]
        message: Option<String>,
    },
}

/// A simple git clone written in Rust
#[derive(Debug, Parser, PartialEq)]
#[command(author = "Chandra Kiran G", version = VERSION, help_template(HELP_TEMPLATE))]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Cmd::Init { name } => {
            let repo_name = name.unwrap_or_else(|| String::from("."));
            initialize_git_dir(Path::new(&repo_name));
        }
        Cmd::Commit { message } => {
            println!("{}", message.unwrap_or_default());
            let git_path = construct_git_path(&Path::new("."));
            let db_path = git_path.join("objects");
            let db = database::Database::new(db_path.to_str().unwrap());
            let workspace = commands::Workspace::new(".");
            let files = workspace.list_files().unwrap();
            let entries = files
                .iter()
                .map(|file| {
                    // println!("File: {}", file);
                    let data = workspace.read_file(file).expect("Error reading file");
                    let mut blob = objects::Blob::new(&data);
                    db.store(&mut blob);
                    entry::Entry::new(&file, &blob.oid.unwrap())
                })
                .collect::<Vec<Entry>>();

            let mut tree = objects::Tree::new(entries);
            db.store(&mut tree);
            println!("{:?}", tree.oid);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use assert_cmd::*;
    use predicates::prelude::*;
    use tempdir::TempDir;

    #[test]
    fn test_construct_git_path() {
        let path = Path::new(".");
        let git_path = construct_git_path(path);
        assert_eq!(
            git_path,
            Path::new(env::current_dir().unwrap().to_str().unwrap()).join(".rgit")
        );

        let path = Path::new("test");
        let git_path = construct_git_path(path);
        assert_eq!(
            git_path,
            Path::new(env::current_dir().unwrap().to_str().unwrap())
                .join("test")
                .join(".rgit")
        );

        let path = Path::new("test/test2");
        let git_path = construct_git_path(path);
        assert_eq!(
            git_path,
            Path::new(env::current_dir().unwrap().to_str().unwrap())
                .join("test")
                .join("test2")
                .join(".rgit")
        );
    }

    #[test]
    fn test_initialize_git_dir() {
        let tmpdir = TempDir::new("test_initialize_git_dir").expect("Failed to create temp dir");
        initialize_git_dir(tmpdir.path());
        let git_path = construct_git_path(tmpdir.path());
        assert!(git_path.exists());

        let tmpdir = TempDir::new("test_initialize_git_dir").expect("Failed to create temp dir");
        fs::create_dir_all(tmpdir.path().join("test")).unwrap();
        initialize_git_dir(&tmpdir.path().join("test"));
        let git_path = construct_git_path(&tmpdir.path().join("test"));
        assert!(git_path.exists());
        assert!(git_path.join("objects").exists());
        assert!(git_path.join("refs").exists());
        assert!(git_path.join("refs").join("heads").exists());
    }

    #[test]
    fn test_initialize_git_dir_with_existing_dir() {
        let tmpdir = TempDir::new("test_initialize_git_dir").expect("Failed to create temp dir");
        fs::create_dir_all(tmpdir.path().join(".rgit")).unwrap();
        initialize_git_dir(tmpdir.path());
        let git_path = construct_git_path(tmpdir.path());
        assert!(git_path.exists());
        assert!(git_path.join("objects").exists());
        assert!(git_path.join("refs").exists());
        assert!(git_path.join("refs").join("heads").exists());

        let tmpdir = TempDir::new("test_initialize_git_dir").expect("Failed to create temp dir");
        fs::create_dir_all(tmpdir.path().join("test").join(".rgit")).unwrap();
        initialize_git_dir(&tmpdir.path().join("test"));
        let git_path = construct_git_path(&tmpdir.path().join("test"));
        assert!(git_path.exists());
        assert!(git_path.join("objects").exists());
        assert!(git_path.join("refs").exists());
        assert!(git_path.join("refs").join("heads").exists());
    }

    #[test]
    fn test_cmd_init() {
        let args = Args::parse_from(&["r_git", "init"]);
        assert_eq!(
            args,
            Args {
                command: Cmd::Init { name: None }
            }
        );

        let args = Args::parse_from(&["r_git", "init", "test"]);
        assert_eq!(
            args,
            Args {
                command: Cmd::Init {
                    name: Some(String::from("test"))
                }
            }
        );

        let mut cmd = Command::cargo_bin("r_git").unwrap();

        cmd.arg("init").assert().success();
        let git_path = construct_git_path(Path::new("."));
        assert!(git_path.exists());
        assert!(git_path.join("objects").exists());
        assert!(git_path.join("refs").exists());
        assert!(git_path.join("refs").join("heads").exists());
        fs::remove_dir_all(git_path).unwrap(); // remove the .rgit directory

        let mut cmd = Command::cargo_bin("r_git").unwrap();
        let argument = "test_dir";
        cmd.arg("init").arg(&argument).assert().success();
        let git_path = construct_git_path(Path::new(argument));
        assert!(git_path.exists());
        assert!(git_path.join("objects").exists());
        assert!(git_path.join("refs").exists());
        assert!(git_path.join("refs").join("heads").exists());
        fs::remove_dir_all(&git_path).unwrap(); // remove the .rgit directory
        fs::remove_dir_all(Path::new(argument)).unwrap(); // remove the test_dir directory

        Command::cargo_bin("r_git")
            .unwrap()
            .arg("init")
            .assert()
            .stdout(
                predicate::str::contains("Initialized empty Git repository in")
                    .and(predicate::str::ends_with(".rgit\n")),
            );
        let git_path = construct_git_path(Path::new("."));
        fs::remove_dir_all(git_path).unwrap(); // remove the .rgit directory

        Command::cargo_bin("r_git")
            .unwrap()
            .arg("init")
            .arg(argument)
            .assert()
            .stdout(
                predicate::str::contains("Initialized empty Git repository in")
                    .and(predicate::str::ends_with(".rgit\n")),
            );
        fs::remove_dir_all(Path::new(argument)).unwrap(); // remove the test_dir directory

        // Test if the .rgit directory is reinitialized
        let mut cmd = Command::cargo_bin("r_git").unwrap();
        cmd.arg("init").assert().success();
        let git_path = construct_git_path(Path::new("."));
        Command::cargo_bin("r_git")
            .unwrap()
            .arg("init")
            .assert()
            .stdout(
                predicate::str::contains("Reinitialized empty Git repository in")
                    .and(predicate::str::ends_with(".rgit\n")),
            );
        fs::remove_dir_all(git_path).unwrap(); // remove the .rgit directory

        let mut cmd = Command::cargo_bin("r_git").unwrap();
        cmd.arg("init").arg(argument).assert().success();
        let git_path = construct_git_path(Path::new(argument));
        Command::cargo_bin("r_git")
            .unwrap()
            .arg("init")
            .arg(argument)
            .assert()
            .stdout(
                predicate::str::contains("Reinitialized empty Git repository in")
                    .and(predicate::str::ends_with(".rgit\n")),
            );

        fs::remove_dir_all(&git_path).unwrap(); // remove the .rgit directory
        fs::remove_dir_all(Path::new(argument)).unwrap(); // remove the test_dir directory
    }
}
