% Getting Started

Before starting playing with Oak, let's install the nightly compiler and create a skeleton project. We are using the [compiler plugins](https://doc.rust-lang.org/book/compiler-plugins.html) extension which is only available in nightly build of Rust. We advise to use the tool [rustup](http://www.rustup.rs) for installing, updating and switching between stable, beta and nightly channels of Rust. The Rust packages manager [Cargo](http://doc.crates.io/) will also be installed with the compiler.

```sh
$ curl https://sh.rustup.rs -sSf | sh
# Switch to nightly build of Rust.
$ rustup default nightly
# Update Rust compiler and Cargo.
$ rustup update
# Switch to stable.
$ rustup default stable
```

For avoiding all compatibility troubles between Oak and the Rust compiler, you should use the version of the Rust compiler matching the one used for compiling Oak. This is done by using `rustup override add <nightly version>` command available in the [README](https://github.com/ptal/oak/).

Once both are installed, we can set up a project using Oak. Run the command `cargo new oak_skeleton` to create a new project. Modify the `Cargo.toml` file to add Oak dependencies:

```
[package]
name = "oak_skeleton"
version = "0.0.1"
authors = ["Pierre Talbot <ptalbot@hyc.io>"]

[dependencies]
oak = "*"
oak_runtime = "*"
```

The `[package]` section describe the usual information about your project, here named *oak_skeleton* and the `[dependencies]` section lists the libraries available on [crates.io](http://crates.io/) that you depend on. You can also directly depend on the git repository:

```
[dependencies.oak]
git = "https://github.com/ptal/oak.git"

[dependencies.oak_runtime]
git = "https://github.com/ptal/oak.git"
path = "runtime"
```

Oak is now usable from your `src/main.rs`:

```rust
#![feature(plugin)]
#![plugin(oak)]

extern crate oak_runtime;
use oak_runtime::*;

grammar! sum{
  #![show_api]

  sum = number ("+" number)* > add
  number = ["0-9"]+ > to_number

  use std::str::FromStr;

  fn add(x: u32, rest: Vec<u32>) -> u32 {
    rest.iter().fold(x, |x,y| x+y)
  }

  fn to_number(raw_text: Vec<char>) -> u32 {
    let text: String = raw_text.into_iter().collect();
    u32::from_str(&*text).unwrap()
  }
}

fn main() {
  let state = sum::parse_sum("7+2+1".into_state());
  assert_eq!(state.unwrap_data(), 10);
}
```

We organized the library into two packages: `oak` and `oak_runtime`. The `oak` dependency is the syntax extension compiling your grammar description into Rust code, the attribute `#![plugin(oak)]` exposes the macro `grammar!` which is the only thing you will use from `oak`. The generated code depends on the library `oak_runtime`, it also contains structures that you will have to use such as `ParseState`. The attribute `#![feature(plugin)]` tells the Rust compiler that we are using unstable features, and that's why we need to use the nightly channel. Keep reading to learn more about the language used in the macro `grammar!`.
