use r_git::workspace_tree::FileOrDir;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("foo/bar/baz");
    let parents = FileOrDir::parent_directories(&path).unwrap();
    println!("{:?}", parents);
}
