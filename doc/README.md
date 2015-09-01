% The Oak Parser Generator

Hello! Oak is a parser generator based on [_Parsing Expression Grammar_ (PEG)](https://en.wikipedia.org/wiki/Parsing_expression_grammar). This project has been started to explore the idea of _typing_ parsing expressions. It is written as a [syntax extension](https://doc.rust-lang.org/book/compiler-plugins.html) and can be embedded in your Rust code without complicating the build system.

Independently of your programming experience with parser generators, a first step is to consult the [Getting Started](getting-started.md) section. If you are new to parser generator or PEG, the section [Learn Oak](learn-oak.md) is a smooth tutorial to Oak for incrementally building a small language with arithmetic expressions and variable bindings. The section [Syntax and Semantics](syntax-and-semantics.md) gives a short summary of the Oak constructions. For more informal and in-depth discussion about the design rational of Oak and to find out why it is different from other parser generators, please consult [The Story of Oak](the-story-of-oak.md).

The code is available on [github](https://github.com/ptal/oak).

### Documentation



### Oak status

My goal is to propose a complete library to ease the development of *Embedded Domain Specific Language* (EDSL) in Rust with procedural macros. For the moment my priority is to stabilize/test things. Next I want to add more static analysis to prevent grammar design error such as in `"=" / "=="` (can you find what is wrong?) Here some other wanted features:

* Automatic wrapping of values into `Spanned<T>` structure to get location information ([#13](https://github.com/ptal/Rust.peg/issues/13)).
* Closest relation between host language types and grammar expression types, for example `e1 > A / e2 > B` with `A` and `B` being variants ([#41](https://github.com/ptal/Rust.peg/issues/41), [#53](https://github.com/ptal/Rust.peg/issues/53), [#54](https://github.com/ptal/Rust.peg/issues/54)).
* Extend the choice operator to handle erroneous cases ([#30](https://github.com/ptal/Rust.peg/issues/30)).
* Bootstrap the grammar ([#42](https://github.com/ptal/Rust.peg/issues/42)).
* Parametrize rules with other rules and arguments ([#10](https://github.com/ptal/Rust.peg/issues/10), [#12](https://github.com/ptal/Rust.peg/issues/12), [#28](https://github.com/ptal/Rust.peg/issues/28)).
* [...](https://github.com/ptal/Rust.peg/issues)

A shortcoming to cleanly achieve these objectives with the Rust compiler is that we can only access item definitions declared inside the procedural macro. It probably means that, for the moment, compositionality would come at the cost of some run-time verifications (or no inter-grammar analysis at all).
