# rgit: A git implementation in Rust

[![codecov](https://codecov.io/gh/kiran-4444/rgit/graph/badge.svg?token=AZR0Q9UL92)](https://codecov.io/gh/kiran-4444/rgit)

This is a simple git implementation in Rust heavily inspired from James Coglan's [Building Git](https://shop.jcoglan.com/building-git/). It is a work in progress and is not yet fully functional. This is in no way a replacement for the original git (though you can use it for toying around), but rather a learning exercise to understand how git works under the hood and to get a better understanding of Rust.

Currently, the following commands are implemented:
1. `rgit init <optional root directory>`: Initialize a new git repository
2. `rgit add <file(s)>`: Add a file(s) to the staging area
3. `rgit commit -m <message>`: Commit the staged files
4. `rgit status`: Show the status of the working directory
5. `rgit diff`: Show the difference between the working directory and the staging area
6. `rgit diff --cached`: Show the difference between the staging area and the last commit

Note: For commit to work, you need to have the following evnironment variables set:
```bash
export RGIT_AUTHOR_NAME="Your Name"
export RGIT_AUTHOR_EMAIL="youremail@gmail.com"
```
This is because the commit command extracts the author name and email from these environment variables.

## Installation

To install `rgit`, clone the repository and run the following command:

```bash
cargo build --release
```

This will create an executable in the `target/release` directory. You can then add this directory to your `PATH` to use `rgit` from anywhere like this:

```bash
export PATH=$PATH:/path/to/rgit/target/release/rgit
```

## Usage

```bash
~/test_rgit git:(master) ✗ rgit
rgit 1.0.0
Chandra Kiran G
A simple git clone written in Rust

Usage:
  rgit <COMMAND>

Commands:
  init    Initialize a new git repository
  commit  Commit the changes in the working tree
  status  Show the working tree status
  add     Add file contents to the index
  diff    Show diff
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Local Development

To run the tests, use the following command:

```bash
cargo test -- --test-threads=1
```

This is necessary because the few tests modify the same repository and run concurrently, which can cause issues. The `--test-threads=1` flag ensures that the tests run sequentially.

