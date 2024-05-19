use colored::Colorize;
use r_git::diff::{EditType, Myres};

fn main() {
    let a = r#"mod author;
mod blob;
mod commit;
mod database;
mod storable;
mod tree;"#;

    let b = r#"pub use self::author::Author;
pub use self::blob::Blob;
pub use self::commit::Commit;
pub use self::database::{Content, Database, FileMode};
pub use self::storable::Storable;"

pub use self::tree::{FlatTree, Tree};"#;

    let myres = Myres::new(a.to_string(), b.to_string());
    let hunks = myres.diff();

    for hunk in hunks {
        let (a_offset, b_offset) = hunk.header();

        let hunks_offsets = format!(
            "@@ -{} +{} @@",
            a_offset
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(","),
            b_offset
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        println!("{}", hunks_offsets.cyan());

        for edit in hunk.edits {
            match edit.edit_type {
                EditType::Add => {
                    println!("{}", format!("+{}", edit.b_line.unwrap().line).green());
                }
                EditType::Remove => {
                    println!("{}", format!("-{}", edit.a_line.unwrap().line).red());
                }
                EditType::Equal => {
                    println!("{}", format!(" {}", edit.a_line.unwrap().line));
                }
            }
        }
    }
}
