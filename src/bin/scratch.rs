use r_git::database::FileMode;

fn main() {
    let mode = FileMode::Regular;
    let mode_int: u32 = mode.into();
    println!("Mode: {}", mode_int);
}
