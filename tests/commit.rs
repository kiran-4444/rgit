use assert_cmd::prelude::*;
use std::process::Command;

// fn setup_git_root_and_move_into_it(base_path: Option<&str>) {
//     let mut cmd = Command::cargo_bin("r_git").unwrap();
//     match base_path {
//         Some(path) => cmd.arg("init").arg(path).assert().success(),
//         None => cmd.arg("init").assert().success(),
//     };

//     std::env::set_var("RGIT_AUTHOR_NAME", "Chandra Kiran G");
//     std::env::set_var("RGIT_AUTHOR_EMAIL", "chandrakiran.g19@gmail.com");

//     Command::new("cd")
//         .arg(base_path.unwrap_or("test_commit"))
//         .assert()
//         .success();

//     Command::new("touch")
//         .arg("test_commit/hello.txt")
//         .assert()
//         .success();

//     Command::new("touch")
//         .arg("test_commit/world.txt")
//         .assert()
//         .success();

//     Command::new("echo")
//         .arg("hello")
//         .arg(">")
//         .arg("test_commit/hello.txt")
//         .assert()
//         .success();

//     Command::new("echo")
//         .arg("world")
//         .arg(">")
//         .arg("test_commit/world.txt")
//         .assert()
//         .success();
// }

#[test]
fn test_commit_without_root_should_fail() {
    let mut cmd = Command::cargo_bin("r_git").unwrap();
    cmd.arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .failure();
    let output = cmd.output().unwrap();

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "fatal: not a git repository (or any of the parent directories): .rgit\n"
    );

    std::fs::remove_dir_all("test_commit").unwrap();
}
