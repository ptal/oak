# Oak

[![ptal on Travis CI][travis-image]][travis]

[travis-image]: https://travis-ci.org/ptal/oak.png
[travis]: https://travis-ci.org/ptal/oak

Compiled on the nightly channel of Rust. Use [rustup](http://www.rustup.rs) for managing compiler channels. You can download and set up the exact same version of the compiler used with `rustup override add nightly-2018-02-26`.

Please consult the [Oak manual](http://hyc.io/oak).

## Features

* Grammar description as a Rust syntax extension.
* Generation of both *recognizer* and *parser* functions for each rules.
* *Type inference* for each parsing expressions. Simplify the AST construction.

## Build local documentation

You might want to build the manual or code documentation from the repository because you need it to be synchronized with a specific version of Oak or simply for offline usage. Here how to do it!

#### Build the manual

You need the utility `rustbook`:

```
git clone https://github.com/steveklabnik/rustbook.git
cd rustbook
cargo build
```

Once built, go inside `oak/doc` and execute `rustup run nigthly <path-to-rustbook>/target/debug/rustbook build`. The manual is generated inside a local folder named `_book`.

#### Build the code documentation

You should be interested by the runtime documentation which is the one useful for users.

```
cd oak/runtime
cargo doc
```

The documentation is available in `oak/runtime/target/doc`.

If you want the developer documentation of the Oak compiler, go to the root of the project and launch:

```
cd oak
rustdoc --no-defaults --passes "collapse-docs" --passes "unindent-comments" --output=target/dev-doc src/liboak/lib.rs
```

The documentation will be available inside `oak/target/dev-doc`.
