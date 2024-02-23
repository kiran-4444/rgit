use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn construct_git_path(path: &Path) -> PathBuf {
    let curr_dir = env::current_dir().expect("Failed to get current directory");

    let creation_path = if path == Path::new(".") {
        curr_dir.to_owned()
    } else {
        curr_dir.join(path).to_owned()
    };

    creation_path.join(".rgit")
}

pub fn initialize_git_dir(path: &Path) {
    let creation_path = construct_git_path(path);
    // Remove the .rgit directory if it exists
    let if_exists = if creation_path.exists() {
        fs::remove_dir_all(&creation_path)
            .map_err(|err| {
                let console_output = format!("Failed to reinitialize git: {}", err);
                eprintln!("{}", console_output.red().bold());
                std::process::exit(1);
            })
            .is_ok()
    } else {
        false
    };

    // Create the .rgit directory with its parent directories
    fs::create_dir_all(&creation_path)
        .map_err(|err| {
            let console_output = format!("Failed to initialize git: {}", err);
            eprintln!("{}", console_output.red().bold());
            std::process::exit(1);
        })
        .ok();

    // Create the objects directory
    fs::create_dir_all(&creation_path.join("objects"))
        .map_err(|err| {
            let console_output = format!("Failed to initialize git: {}", err);
            eprintln!("{}", console_output.red().bold());
            std::process::exit(1);
        })
        .ok();

    // Create the refs directory
    fs::create_dir_all(&creation_path.join("refs"))
        .map_err(|err| {
            let console_output = format!("Failed to initialize git: {}", err);
            eprintln!("{}", console_output.red().bold());
            std::process::exit(1);
        })
        .ok();

    // Create the refs/heads directory
    fs::create_dir_all(&creation_path.join("refs").join("heads"))
        .map_err(|err| {
            let console_output = format!("Failed to initialize git: {}", err);
            eprintln!("{}", console_output.red().bold());
            std::process::exit(1);
        })
        .ok();

    // Give the user a nice message
    let console_output = if if_exists {
        format!(
            "Reinitialized empty Git repository in {}",
            creation_path.to_str().unwrap()
        )
    } else {
        format!(
            "Initialized empty Git repository in {}",
            creation_path.to_str().unwrap()
        )
    };

    println!("{}", console_output.green());
}
