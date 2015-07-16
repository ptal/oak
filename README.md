Rust.peg
========

Declare &amp; Use a grammar directly in your code.

My goals are:

* Assisted AST generation by typing the rules.
* Generate only code that doesn't compile to errors.
* Grammars that can be composed together to be re-used in other parser.

This library is highly experimental, I don't even know if my goals are possible.

What I would like to achieve before 1.0 and publishing on crates.io:

- [ ] Semantics actions. The code generation is missing but most of the type inference on rules is done. The "big idea" is to avoid naming expression in the rule (such as `e1:v1`) and to keep the grammar as clean as possible.
- [ ] Decent error reporting. Probably something based on [1].
- [ ] It definitely needs more tests.

[1] André Murbach Maidl, Sérgio Medeiros, Fabio Mascarenhas, and Roberto Ierusalimschy. [Error reporting in parsing expression grammars](http://arxiv.org/abs/1405.6646). arXiv preprint arXiv:1405.6646, 2014.
