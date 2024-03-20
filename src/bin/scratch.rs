macro_rules! test {
    ($i:ident, $x:expr) => {
        $i = $x + 1;
    };
}

fn main() {
    println!("Hello, world");
    let mut x = 5;
    test!(x, 10);
    println!("x = {}", x);
}
