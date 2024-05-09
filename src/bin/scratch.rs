use r_git::utils::decompress_content;

fn main() {
    let oid = "2f34eabf28a920ac2a62ccfdc3cd080cfdeb4314";
    let content = decompress_content(oid).unwrap();
    println!("{}", content);
}
