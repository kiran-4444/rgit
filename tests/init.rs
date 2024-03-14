use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn test_init_with_no_args() {
    let mut cmd = Command::cargo_bin("r_git").expect("Failed to build binary");
    cmd.arg("init").assert().success();

    // check if the .rgit directory is created
    assert!(std::path::Path::new(".rgit").exists());

    // check if objects directory is created
    assert!(std::path::Path::new(".rgit/objects").exists());

    // remove the .rgit directory
    std::fs::remove_dir_all(".rgit").expect("Failed to remove .rgit directory");
}

#[test]
fn test_init_with_args() {
    let mut cmd = Command::cargo_bin("r_git").expect("Failed to build binary");
    cmd.arg("init").arg("sub_path").assert().success();

    // check if the .rgit directory is created
    assert!(std::path::Path::new("sub_path/.rgit").exists());

    // check if objects directory is created
    assert!(std::path::Path::new("sub_path/.rgit/objects").exists());

    // remove the .rgit directory
    std::fs::remove_dir_all("sub_path/").expect("Failed to remove .rgit directory");
}
