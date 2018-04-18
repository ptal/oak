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

You need the utility [mdbook](https://rust-lang-nursery.github.io/mdBook/):

```
cargo install mdbook
```

Once installed, go inside `oak/doc` and execute `mdbook build -o`.
The manual is generated inside a local folder named `book` and directly opened in your browser.

#### Build the code documentation

As a user of Oak, you will be interested by the runtime documentation.

```
cd oak/runtime
cargo doc
```

The documentation is then available in `oak/runtime/target/doc`.

To build the internal documentation of Oak, you can type this command at the root of the project:

```
cd oak
rustdoc --document-private-items --output=target/dev-doc src/liboak/lib.rs
```

The documentation will be available inside `oak/target/dev-doc`.
It is useful to work on Oak :-)
