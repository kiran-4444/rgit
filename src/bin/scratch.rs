use std::path::PathBuf;

use walkdir::WalkDir;

fn main() {
    let files = WalkDir::new(".")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_type()
                .is_file()
                .then(|| entry.path().to_path_buf())
        })
        .collect::<Vec<PathBuf>>();

    for file in files {
        println!("{:?}", file);
    }
}
