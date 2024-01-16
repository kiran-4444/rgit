use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;
use std::{env, fs};

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
}

/// A simple git clone written in Rust
#[derive(Debug, Parser, PartialEq)]
#[command(author = "Chandra Kiran G", version = VERSION, help_template(HELP_TEMPLATE))]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

fn initialize_git_dir(path: &Path) {
    let curr_dir = env::current_dir().expect("Failed to get current directory");
    let creation_path = if path == Path::new(".") {
        curr_dir
    } else {
        curr_dir.join(path)
    }
    .join(".rgit");

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

fn main() {
    let args = Args::parse();

    match args.command {
        Cmd::Init { name } => {
            let repo_name = name.unwrap_or_else(|| String::from("."));
            initialize_git_dir(Path::new(&repo_name));
        }
    }
}
