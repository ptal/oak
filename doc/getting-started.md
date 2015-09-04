% Getting Started

Before starting playing with Oak, let's create a skeleton project. There are some requirements:

* We are using the [compiler plugins](https://doc.rust-lang.org/book/compiler-plugins.html) extension that is only available in nightly build of Rust. It is simple to install but you need to build from source the Rust compiler. Instructions are available in the [Rust book](http://doc.rust-lang.org/book/nightly-rust.html).
* The Rust packages manager [Cargo](http://doc.crates.io/) is highly recommended. If you followed the Rust book for installing the nightly build, you have nothing to do! Cargo comes installed with the Rust compiler.

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

  sum = value ("+" value)* > add
  value = ["0-9"]+ > to_digit

  fn add(x: u32, rest: Vec<u32>) -> u32 {
    rest.iter().fold(x, |x,y| x+y)
  }

  fn to_digit(raw_text: Vec<char>) -> u32 {
    use std::str::FromStr;
    let text: String = raw_text.into_iter().collect();
    u32::from_str(&*text).unwrap()
  }
}

fn main() {
  let state = sum::parse_sum("7+2+1".stream());
  assert_eq!(state.unwrap_data(), 10);
}
```

We organized the library into two packages: `oak` and `oak_runtime`. The `oak` dependency is the syntax extension compiling your grammar description into Rust code, the attribute `#![plugin(oak)]` exposes the macro `grammar!` which is the only thing you will use from `oak`. The generated code depends on the library `oak_runtime`, it also contains structures that you will have to use such as `ParseState`. The attribute `#![feature(plugin)]` tells the Rust compiler that we are using unstable features, and that's why we need to use the nightly channel. Keep reading to learn more about the language used in the macro `grammar!`.
