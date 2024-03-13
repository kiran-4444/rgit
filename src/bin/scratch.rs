use std::path::Path;

fn main() {
    let path = Path::new("a/b/c/d.txt");
    let result = get_all_prefixes(path);
    println!("{:?}", result);
}

fn get_all_prefixes(path: &Path) -> Vec<String> {
    let mut prefixes = Vec::new();
    let parts: Vec<&str> = path
        .components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect();
    let mut current_path = String::new();
    for part in parts.iter().take(parts.len() - 1) {
        current_path.push_str(part);
        prefixes.push(current_path.to_owned());
        current_path.push('/');
    }
    prefixes
}
