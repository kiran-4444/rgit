use std::path::Path;

fn main() {
    let metadata = std::fs::metadata(Path::new("Justfile"));
    println!("{:?}", metadata);
}
