% The Oak Parser Generator

Hello! Oak is a parser generator based on [_Parsing Expression Grammar_ (PEG)](https://en.wikipedia.org/wiki/Parsing_expression_grammar). This project has been started to explore the idea of _typing_ parsing expressions. It is written as a [syntax extension](https://doc.rust-lang.org/book/compiler-plugins.html) and can be embedded in your Rust code without complicating the build system.

Independently of your programming experience with parser generators, a first step is to consult the [Getting Started](getting-started.md) chapter. If you are new to parser generator or PEG, the chapter [Learn Oak](learn-oak.md) is a smooth tutorial to Oak for incrementally building a small language named `Calc` with arithmetic expressions and variable bindings. You can directly dive into the full grammar of `Calc` in the chapter [Full Calc Grammar](full-calc-grammar.md). If you want to learn about the Oak specificities, please go to the chapter [Typing Expression](typing-expression.md). Finally, in the chapter [Related Work](related-work.md), we compare Oak to existing parser generators and give some references and credits to papers or implementations that inspired the design of Oak.

The code is available on [github](https://github.com/ptal/oak).

### Documentation

* [Oak manual](http://hyc.io/oak) – current page.
* [Oak runtime documentation](http://hyc.io/oak_runtime)

### Syntax cheat sheet

`e` is a sub expression and `T` is the type of `e`. The types are only informative, it does not show unit propagation, more in [Typing Expression](typing-expression.md). Greedy operators do not generate "backtracking points" and consume as many characters as possible.

| Expression      | Type                  | Precedence level | Description |
| --------------- | --------------------- |----------------- | ----------- |
| `"literal"`     | `(^)`                 | 0                | Match a string literal. |
| `.`             | `char`                | 0                | Match any single character. |
| `["a-zA-Z-"]`   | `char`                | 0                | Match a character from one of the specified classes. |
| `(e)`           | `T`                   | 0                | Group an expression. |
| `ident`         | Type of rule `ident`  | 0                | Call the rule with the name `ident`. |
| `e?`            | `Option<T>`           | 1                | (Greedy) Match zero or one `e`. Always succeed. |
| `e*`            | `Vec<T>`              | 1                | (Greedy) Match zero or more `e`. Always succeed. |
| `e+`            | `Vec<T>`              | 1                | (Greedy) Match one or more `e`. |
| `&e`            | `(^)`                 | 2                | Try to match `e` and succeed if `e` succeeds. It does not consume any input. |
| `!e`            | `(^)`                 | 2                | Try to match `e` and succeed if `e` fails. It does not consume any input. |
| `e1 e2 e3`      | `(T1, T2, T3)`        | 3                | Match `e1 e2 e3` in sequence. Immediately fails when one fails. |
| `e > f`         | Return type of `f`    | 4                | Match `e` and if it succeeds, call `f(v)` where `v` is the value of `e`. |
| `e -> ()`       | `()`                  | 4                | Force the type of `e` to be `()`. |
| `e -> (^)`      | `(^)`                 | 4                | Force the type of `e` to be `(^)`. |
| `e1 / e2 / e3`  | Type of any `e`       | 5                | Match `e1 e2 e3` in sequence. Immediately succeeds when one succeeds. |

### Oak status

My goal is to propose a complete library to ease the development of *Embedded Domain Specific Language* (EDSL) in Rust with procedural macros. For the moment my priority is to stabilize and test Oak. Next I want to add more static analysis to prevent grammar design error such as in `"=" / "=="` (can you find what is wrong?) Here some other wanted features:

* Automatic wrapping of values into `Spanned<T>` structure to get location information ([#13](https://github.com/ptal/Rust.peg/issues/13)).
* Closest relation between host language types and grammar expression types, for example `e1 > A / e2 > B` with `A` and `B` being variants ([#41](https://github.com/ptal/Rust.peg/issues/41), [#53](https://github.com/ptal/Rust.peg/issues/53), [#54](https://github.com/ptal/Rust.peg/issues/54)).
* Extend the choice operator to handle erroneous cases ([#30](https://github.com/ptal/Rust.peg/issues/30)).
* Bootstrap the grammar ([#42](https://github.com/ptal/Rust.peg/issues/42)).
* Parametrize rules with other rules and arguments ([#10](https://github.com/ptal/Rust.peg/issues/10), [#12](https://github.com/ptal/Rust.peg/issues/12), [#28](https://github.com/ptal/Rust.peg/issues/28)).
* [...](https://github.com/ptal/Rust.peg/issues)

A shortcoming to cleanly achieve these objectives with the Rust compiler is that we can only access item definitions declared inside the procedural macro (that's true, isn't it?). It probably means that, for the moment, compositionality would come at the cost of some run-time verifications (or no inter-grammar analysis at all).
