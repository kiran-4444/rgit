fn main() {
    let a: Vec<u8> = vec![120, 156, 165, 143, 65, 10, 194, 48, 16];

    let a_str = String::from_utf8(a).unwrap();
    println!("{:?}", a_str);
}
