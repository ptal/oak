% Learn Oak

This section is devoted to introduce smoothly the different PEG combinators through a tutorial presenting `Calc`: a small language with arithmetic expressions and variable bindings. If you want to test the code while reading this tutorial, a skeleton project is available in the section [Getting Started](getting-started.md). Before diving into the details, we present a program written in `Calc`:

```
let a = 10 - 2 in
let b = a / 2 in
a + 3 * (b / 2)
```

It declares two local variables `a` and `b` initialized with arithmetic expressions and usable within the scope of the let-binding, which is everything after the `in`. Let-bindings can be composed in cascade but must terminates with an arithmetic expression, such as `a + 3 * (b / 2)` in our example.

### What is parsing?

A parser is a bridge between meaning-less sequence of characters and structured representation of data. It tries to give meanings to raw characters by constructing an *Abstract Syntax Tree* (AST) that will be processed by subsequent compilation phases. We expect a parser to transform `7 - 1` into a structure such as `Minus(i32, i32)`. As a side note, you should avoid to compute the actual result of `7 - 1` in the parsing step, it works for simple language but tends to entangle syntactic and semantic analysis later. Invalid programs such as `let a = 8 in a * b` will still be correctly parsed, the semantic analysis is responsible for detecting that `b` is undeclared.

This tutorial will not cover the semantic analysis part and will only describe the grammar used for parsing `Calc`. Our parser will thus produce an AST and will not evaluate expressions.

### Syntactic atoms of `Calc`

When it comes to elaborate a grammar, we usually start by identifying atoms of the language, e.g. syntactic constructions that can not be divided into smaller ones. These atoms are called *tokens* and are often processed during a *lexical analysis* happening before the parsing. Oak is based on _Parsing Expression Grammar_ (PEG) and works directly on a stream of characters instead of a stream of tokens. An advantage is to have a unique and coherent grammar syntax which is helpful for composing grammars that do not necessarily expect the same set of tokens. Before continuing reading, try to find out what are the atoms of `Calc`.

The keywords `let` and `in`, the binding operator `=`, parenthesis `()` and arithmetic operators `+`, `-`, `*`, `/` form the *unvalued atoms* of the language. `Calc` has two *valued atoms* which are identifiers and integers. Unvalued atoms give a shape to the AST but they do not carry any specific data retrieved from the stream of characters. The following grammar parses the atoms of `Calc`:

```
grammar! calc {
  #![show_api]

  let_kw = "let"
  in_kw = "in"
  bind_op = "="
  add_op = "+"
  sub_op = "-"
  mul_op = "*"
  div_op = "/"

  identifier = ["a-zA-Z0-9_"]+
  integer = ["0-9"]+
}
```

A grammar is introduced with the macro `grammar! <name>` where `<name>` is the name of the grammar but also the name of the module in which generated functions will lie. A grammar is a set of rules of the form `<name> = <expr>` where `<name>` is the rule name and `<expr>` is a parsing expression.

The rules describing keywords and operators use *string literals* expressions of the form `"<literal>"`, it expects the input to match exactly the sequence of characters given.

Identifiers and integers are recognized with *character classes* where a class is a single character or a character range. A range `r` has the form `<char>-<char>` inside a set `["r1r2..rN"]`. Since `-` is used to denote a range, it must be placed before or after all the ranges such as in `["-a-z"]` to be recognized as an accepted character. Character classes will succeed and "eat" *one* character if it is present in the set, so `b`, `8`, `_` are all accepted by `["a-zA-Z0-9_"]` but `é`, `-` or `]` are not.

For both string literals and character classes, any Unicode characters are interpreted following the same requirements as [string literals](https://doc.rust-lang.org/reference.html#string-literals) in the Rust specification. The only other parsing expression consuming a character is the `.` expression, it matches any character and can only fail if we reached the end of input.

The remaining parsing expressions are combinators, they must be composed with sub-expressions. Identifiers and integers are sequences of one or more characters and we use the combinator `e+` to repeat `e` while it succeeds. For example `identifier` matches "x_1" from the input "x_1 x_2" by successively applying `["a-zA-Z0-9_"]` to the input; it parses `x`, `_` and `1` and then fails on the space character. It however succeeds, even if the match is partial, and `identifier` returns the remaining input " x_2" and the data read. A requirement of `e+` is that `e` must be repeated *at least once*. The `e*` expression does not impose this constraint and allow `e` to be repeated *zero or more times*. The last combinator in this category is `e?`, it consumes `e` *zero or one time*. The combinators `e*`, `e+` and `e?` will consume as much input as they can and are said to be *greedy operators*.

### Generated code and runtime

Before explaining the others combinators, we take a glimpse at the generated code and how to use it. Oak will generate two functions per rule, a *recognizer* and a *parser*. A recognizer only matches the input against a specific rule but does not build any value from it. A parser matches and builds the corresponding AST (possibly with the help of user-specific functions called *semantic actions*). For example, the functions `parse_identifier` and `recognize_identifier` will be generated for rule `identifier`. The `#![show_api]` attribute tells Oak to output, as a compilation note, the signatures of all the generated functions. We obtain the following from the `Calc` grammar:

```rust
// `ParseState` and `CharStream` are prefixed by `oak_runtime::`.
// It is removed from this snippet for clarity.
note: pub mod calc {
    pub fn parse_let_kw<S>(mut stream: S) -> ParseState<S, ()>
     where S: CharStream;
    pub fn recognize_let_kw<S>(mut stream: S) -> ParseState<S, ()>
     where S: CharStream;

    pub fn parse_identifier<S>(mut stream: S) -> ParseState<S, Vec<char>>
     where S: CharStream;
    pub fn recognize_identifier<S>(mut stream: S) -> ParseState<S, ()>
     where S: CharStream;

    pub fn parse_integer<S>(mut stream: S) -> ParseState<S, Vec<char>>
     where S: CharStream;
  // ...
  // Rest of the output truncated for the tutorial.
}
```

We can already use these functions in our main:

```rust
fn main() {
  let let_kw = "let";
  let state = calc::recognize_let_kw(let_kw.stream());
  assert!(state.is_successful());

  let ten = "10";
  let state = calc::parse_integer(ten.stream());
  assert_eq!(state.unwrap_data(), vec!['1', '0']);
}
```

First of all, there is a [documentation of the runtime](http://hyc.io/oak_runtime) available, but please, be aware that it also contains functions and structures used by the generated code that you will probably not need.

Parsing functions accept a stream as input parameter which represents the data to be processed. A stream can be retrieved from type implementing `Stream` with the method `stream()` which is similar to `iter()` for retrieving an iterator. For example, `Stream` is implemented for the type `&'a str` and we can directly pass the result of `stream()` to the parsing function, as in `calc::recognize_let_kw(let_kw.stream())`. Basically, a stream must implement several operations described by the `CharStream` trait, it is generally an iterator that keeps a reference to the underlying data traversed. You can find a list of all types implementing `Stream` in the [implementors list of `Stream`](http://hyc.io/rust-lib/oak/oak_runtime/stream/trait.Stream.html).

By looking at the signatures of `parse_identifier` and `recognize_identifier` we see that a value of type `ParseState<S, T>` is returned. `T` is the type of the data extracted during parsing. It is always equal to `()` in case of a recognizer since it does not produce data, and hence a recognizer is a particular case of a parser where the AST has type `()`. In the rest of this tutorial and when not specified, we consider the term *parser* to also include recognizer.

A state indicates if the parsing was successful, partial or erroneous. It carries information about which item was expected next and the AST built from the data read. Convenient functions such as `unwrap_data()` or `is_successful()` are available directly from [ParseState](http://hyc.io/rust-lib/oak/oak_runtime/parse_state/struct.ParseState.html). A more complete function is `into_result()` which transforms the state into a type `Result` that can be pattern matched. Here a full example:

```rust
fn analyse_state(state: ParseState<StrStream, Vec<char>>) {
  match state.into_result() {
    Ok((success, error)) => {
      if success.partial_read() {
        println!("Partial match: {:?} because: {}", success.data, error);
      }
      else {
        println!("Full match: {:?}", success.data);
      }
    }
    Err(error) => {
      println!("Error: {}", error);
    }
  }
}

fn main() {
  analyse_state(calc::parse_integer("10".stream())); // complete
  analyse_state(calc::parse_integer("10a".stream())); // partial
  analyse_state(calc::parse_integer("a".stream())); // erroneous
}

// Result:

// Full match: ['1', '0']
// Partial match: ['1', '0'] because: 1:3: unexpected `a`, expecting `["0-9"]`.
// Error: 1:1: unexpected `a`, expecting `["0-9"]`.
```

You are now able to efficiently use the code generated by Oak.

### Semantic action

As you probably noticed, the rule `integer` produces a value of type `Vec<char>` which is not a usable representation of an integer. We must transform this value into a better type such as `u32`. To achieve this goal, we use a *semantic action* which gives meaning to the characters read. A semantic action is a Rust function taking the value produced by an expression and returning another one more suited for further processing. The grammar becomes:

```rust
grammar! calc {
  #![show_api]

  // ... previous rules truncated.

  integer = ["0-9"]+ > to_digit

  pub type Digit = u32;

  fn to_digit(raw_text: Vec<char>) -> Digit {
    use std::str::FromStr;
    let text: String = raw_text.into_iter().collect();
    u32::from_str(&*text).unwrap()
  }
}
```

The combinator `e > f` expects a parsing expression on the left and a function name on the right, it works like a "reverse function call operator" in the sense that `f` is called with the result value of `e`. Semantic actions must be Rust functions declared inside the `grammar!` so we can examine its return type. You can call function from other modules or crates by wrapping it up inside a function local to the grammar. Any Rust code is accepted, here we use an extra type declaration `Digit` which will be accessible from outside with `calc::Digit`.

Oak gives a type to any parsing expression to help you constructing your AST more easily. Next sections explain how Oak gives a type to expressions and how you can help Oak to infer better types. For the moment, when you want to know the type of an expression, just creates a rule `r = e`, activates the attribute `#[show_api]` and consults the return type of the generated function from the compiler output. Note that a tuple type such as `(T, U)` is automatically unpacked into two function arguments, so we expect the function to be of type `f(T, U)` and not `f((T, U))`.

### Choice combinator

We can now build another part of our language: a simple arithmetic calculator where operands can be integers or variables (identifiers). We can extend our grammar with a `factor` rule:

```rust
grammar! calc {
  #![show_api]

  // ... previous rules and code truncated.

  factor
    = integer > digit_expr
    / identifier > variable_expr

  pub type PExpr = Box<Expression>;

  pub enum Expression {
    Variable(String),
    Digit(u32)
  }

  fn digit_expr(digit: u32) -> PExpr {
    Box::new(Digit(digit))
  }

  fn variable_expr(raw_text: Vec<char>) -> PExpr {
    Box::new(Variable(raw_text.into_iter().collect()))
  }
}
```

A new combinator appeared! Indeed, an operand can be an `integer` *or* an `identifier` (for variables) and this alternative is expressed with the *choice combinator* of the form `e1 / e2 / ... / eN`. It tries the expression `e1` and if it fails, it restarts with `e2`, etc. It fails if the last expression `eN` fails. An important point is that *order* matters, hence the grammar is unambiguous, for each input, only one parse tree is possible. It's worth mentioning that this prioritized choice can leads to unexpected, but however easy to detect, wrong behaviour. For example, if you consider `identifier / integer` which reverses the order of the factors, `integer` will never be reached because `identifier` accepts a super-set of the language recognized by `integer`. Choice combinators naturally map to an enumeration type in Rust, in our example we declared `Expression` within the macro. We build the variants of the enumeration with our own functions. Note that types can be declared outside the macro, you just need to add the corresponding `use` statements.

### Sequence combinator

We have all the pieces to parse our first arithmetic expression. We start with `+` and `-` because they have the same precedence, we will next add `*` and `/`. The sequence combinator is required to parse a sequence of two or more PEGs and is denoted as `e1 e2 ... eN`. If `e1` succeeds, then `e2` is called and so on until `eN` succeeds. It fails if one fails, this is the main difference from the choice combinator which fails if the last expression fails. Let's give a look to the new grammar:

```rust
grammar! calc {
  #![show_api]

  // ... previous rules and code truncated.

  term_op
    = add_op > add_bin_op
    / sub_op > sub_bin_op

  expression
    = factor (term_op factor)* > fold_left

  use self::Expression::*;
  use self::BinOp::*;

  pub type PExpr = Box<Expression>;

  pub enum Expression {
    Variable(String),
    Digit(u32),
    BinaryExpr(BinOp, PExpr, PExpr)
  }

  pub enum BinOp {
    Add,
    Sub
  }

  fn fold_left(head: PExpr, rest: Vec<(BinOp, PExpr)>) -> PExpr {
    rest.into_iter().fold(head,
      |accu, (op, expr)| Box::new(BinaryExpr(op, accu, expr)))
  }

  fn add_bin_op() -> BinOp { Add }
  fn sub_bin_op() -> BinOp { Sub }
}
```

Parsing rules for arithmetic expression are usually written with *left recursion* which would give us a rule such as:

```rust
expression
  = expression term_op factor
  / factor
```

Grammar descriptions written in PEG are closed to hand-written recursive descent parser while context-free language specification are less-tied to implementation. This is why left recursion often leads to infinite loops (and eventually to stack overflow) in PEG implementation while it is nicely handled in other parser generator. Oak does not support left recursion yet so the grammar above will generate invalid code. However, we wrote the first `expression` rule without left recursion, it is possible if you see an expression as a *list* of factors separated by binary operators. The tip is to handle repetition with the `e*` combinator instead of recursive rules.

Due to the lack of left recursion, the shape of the tree is modified and this is why we use the function `fold_left` to create a binary tree from a list of expression. This kind of structure is more convenient for the semantic analysis but it is possible to use a variant for expression list such as `ExprList(PExpr, Vec<(BinOp, PExpr)>)`.

### Operator precedence

### Syntactic predicates

### Spacing

### Operator associativity

### Exercises

* Extend the grammar to support negative integers.
* Extend the grammar to support `let-in` anywhere in expressions. Note that you do not need to modify the AST structure.
