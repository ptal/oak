# Oak

## Super quick example

```rust
grammar! calculator{
  #![show_api]

  expression = sum

  sum
    = product ("+" product)* > add

  product
    = value ("*" value)* > mult

  value
    = ["0-9"]+ > to_digit
    / "(" expression ")"

  pub type Digit = u32;

  fn add(x: Digit, rest: Vec<Digit>) -> Digit {
    rest.iter().fold(x, |x,y| x+y)
  }

  fn mult(x: Digit, rest: Vec<Digit>) -> Digit {
    rest.iter().fold(x, |x,y| x*y)
  }

  fn to_digit(raw_text: Vec<char>) -> Digit {
    use std::str::FromStr;
    let text: String = raw_text.into_iter().collect();
    u32::from_str(&*text).unwrap()
  }
}

fn main() {
  assert_eq!(calculator::parse_expression("7+(7*2)", 0).unwrap().data, 21);
}
```

## Syntax cheat sheet

`e` is a sub expression and `T` is the type of `e`. The types are only informative, it does not show unit propagation. Greedy operators do not generate "backtracking points" and consume as many characters as possible.

| Expression      | Type                  | Precedence level | Description |
| --------------- | --------------------- |----------------- | ----------- |
| `"literal"`     | `()`                  | 0                | Match a string literal. |
| `.`             | `char`                | 0                | Match any single character. |
| `["a-zA-Z-"]`   | `char`                | 0                | Match a character from one of the specified classes. |
| `(e)`           | `T`                   | 0                | Group an expression. |
| `ident`         | Type of rule `ident`  | 0                | Call the rule with the name `ident`. |
| `e?`            | `Option<T>`           | 1                | (Greedy) Match zero or one `e`. Always succeed. |
| `e*`            | `Vec<T>`              | 1                | (Greedy) Match zero or more `e`. Always succeed. |
| `e+`            | `Vec<T>`              | 1                | (Greedy) Match one or more `e`. |
| `&e`            | `(^)`                 | 2                | Try to match `e` and succeed if `e` succeeds. It does not consume any input. |
| `!e`            | `(^)`                 | 2                | Try to match `e` and succeed if `e` fails. It does not consume any input. |
| `e -> ()`       | `()`                  | 3                | Force the type of `e` to be `()`. |
| `e -> (^)`      | `(^)`                 | 3                | Force the type of `e` to be `(^)`. |
| `e1 e2 e3`      | `(T1, T2, T3)`        | 4                | Match `e1 e2 e3` in sequence. Immediately fails when one fails. |
| `e > f`         | Return type of `f`    | 5                | Match `e` and if it succeeds, call `f(v)` where `v` is the value of `e`. |
| `e1 / e2 / e3`  | Type of any `e`       | 6                | Match `e1 e2 e3` in sequence. Immediately succeeds when one succeeds. |

## Introduction to expressions types and `(^)`

The full explanation of the what and why of types is available in the next section. A goal of this library is to give a type to any expression grammar. This permits to call a semantic action without naming expressions. In some cases, for example with the spacing rule `spacing = [" \n\t"]*`, the grammar compiler will not generate the expected type, here the rule `spacing` has type `Vec<char>` instead of `()` — we usually do not care about spaces. Therefore, users must annotate expressions with `e -> ()` to force their types to be `()`. It works and is enough for most cases. However, we sometimes want to propagate unit type up in the expression tree because these expressions are only of syntactic interest. The fact is that `e?` has type `Option<T>` even if `T = ()`. It is expected since `Option<()>` carries a boolean information about the presence of something. If we do not care, we can annotate `e` with `(^)` and the unit type will automatically be propagated, and even `e1? e2*` will have type `(^)` if `e1` and `e2` have type `(^)`. In the end, the goal is really to give an expression the type that you expect it to have!

## The story of Oak

_This section explains the novelties of this library and is not a tutorial to parsing expression grammar. It supposes the reader knowledgeable on the subject so if you are not, you can consult tutorial on Parsing Expression Grammar on the internet. We might provide one later._

I started this project because I had the idea of _typing_ grammar rules. It comes from the observation that _Parsing Expression Grammar_ (PEG) combinators are really close to traditional types: choice is a sum type, sequence is a tuple, repetition is an array,... I wondered if we could automatically generate an AST from a grammar description so both would be automatically kept in sync. It turned out that generating the AST (data type included) was hard, mostly because we need to _name_ things and rules does not give enough information by themselves – how to name the variants of the sum type? Of course, we could annotate the expressions with names but I wanted to keep the grammar syntax as close as possible to what you could find in a text-book.

In traditional parser generators, the bridge between purely syntactic analysis and AST construction is done with semantic actions. Interaction between an expression and the user code is usually done with one of these two techniques (`digit` being a rule parsing an integer):

1. Positional arguments: `digit "+" digit { $$ = $1 + $3; }` is a technique used in [Yacc](http://dinosaur.compilertools.net/yacc/) for example.
2. Expression labelling: `digit:x "+" digit:y { x + y }` is similar to what is used in [Menhir](http://gallium.inria.fr/~fpottier/menhir/) (parser generator written in OCaml).

The first technique is often discouraged because some errors can silently appear if you change the order of expression inside a rule without changing the associated action or if you make a mistake when numbering the arguments. The generated code will fail to compile if the host language is statically typed and if the two expressions have different types, but in the general case this technique is not safe. Expression labelling is better but it has the inconvenient of burdening the grammar syntax. Also note that none of these techniques help the user to build the corresponding AST, their purposes is to offer a simple interface between grammar and host code.

Using the idea of typing grammar, we can give a type to each expression and directly pass the value to the semantic action without any labelling or positional notation. The previous example becomes `digit "+" digit > add` with `>` being a "reverse function call operator", the expression `digit "+" digit` produces a value `v` of type `(i32, i32)` and the code generated looks like `add(v)`. It is even smarter and will automatically unpack the tuple into function arguments, so the function `add` will be called with two arguments of type `i32`.

Cool isn't it? However the implementation was more complicated than I first thought due to expressions of type unit (`()`). Consider this grammar for parsing simple variable identifiers:

```
var_ident = !["0-9"] ["a-zA-Z0-9_"]+ spacing
spacing = [" \n\t"]*
```

In rule `var_ident` the only value of interest is the one returned by the expression `["a-zA-Z0-9_"]+` and it has type `Vec<char>` (note that we could use a semantic action to transform this value into a string). It is natural to think that the rule `var_ident` will be of type `Vec<char>` too. However, a trivial algorithm infers that this expression has type `(char, Vec<char>, Vec<char>)`, the sequence has three sub expressions and thus it forms a 3-tuple. We do not care about the value of the spaces but this is not something that a computer can guess by itself so we must force the type to be unit using the arrow operator:

```
var_ident = !["0-9"] ["a-zA-Z0-9_"]+ spacing
spacing = [" \n\t"]* -> ()
```

This operator is the only annotation not directly related to parsing that we need. The new type of `var_ident` is now `(char, Vec<char>, ())`. The generator automatically reduce this type to `Vec<char>` thanks to a few rules:

* Everything under a syntactic predicate (`!e` or `?e`) has type `()`. The new type is `((), Vec<char>, ())`.
* Any unit type inside a tuple is removed. We now have `(Vec<char>)`.
* Type inside a 1-tuple is extracted. We finally obtain `Vec<char>`.

These _type rewriting rules_ are intuitive because they produce the type the user expects! Type annotation is only needed to specify that we are not interested by the value, such as with spaces.

Implementation is not that easy due to the fact that some rules have recursive types. For example, the following grammar accept strings for which any character at position `i` is 'a' or 'b' if `i` is even and otherwise 'c' or 'd'.

```
a_b = ["a-b"] c_d?
c_d = ["c-d"] a_b?
```

This is a totally valid grammar but it can not be typed without recursive types. Let's try anyway. Say that `a_b` has type `T` and `c_d` has type `U`. We can infer that `T = (char, Option<U>)` for rule `a_b` and `U = (char, Option<T>)` for rule `c_d`. By substitution we get `T = (char, Option<(char, Option<T>))` and thus `T` is defined by itself. It is a recursive type definition. You might think that this type is fine as long as we give an alias to the tuple types:

```rust
type T = (char, Option<U>);
type U = (char, Option<T>);
```

However the names `T` and `U` are completely arbitrary and the user probably do not want types with random names. We would need name-annotations on expressions which is not our leitmotiv in the first place. It is cleaner and easier to let the user constructs the types by himself with semantic actions. Furthermore, here the type `Option<U>` introduces the necessary indirection for building recursive types but what if the grammar does not explicitly give one? We should automatically infer a type supporting recursive type which seems a lot of work for little benefit. In the end, we forbid implicit recursive types and let the user write semantic actions to build them. However recursive rules are still allowed. The difficulty was to generate a compile-time error only in the case we care about the value produced by the recursive rules. Actually, the previous grammar will be accepted by the parser generator if we are using the rule `a_b` below a syntactic predicate, for example the expression `!a_b` implies that we do not need to build the value of `a_b`.

A new quirk appears if we want to propagate and erase unit type in expression to obtain a smaller type (for example, `((), T)` is reduced to `T`). If we consider the following grammar which describe the optional presence of the `mut` keyword on the left-hand side of a let-expression, everything works fine!

```
let_left = let_kw mut_kw? var_ident
let_kw = "let" spacing
mut_kw = "mut" spacing
```

Indeed, the type of the expression `mut_kw?` is `Option<()>`. This is expected since the type `Option<()>` carries a boolean information. As a rule of thumb, unit inference never erase a potential piece of information. A problematic case is the following simplified grammar describing the optional keyword `return` in the last statement of a function:

```
return_expr = return_kw? expr
return_kw = "return" spacing
```

In this particular case we do not care if the keyword `return` is present or not, it is only of syntactic interest. Using what we already know, we can use a type annotation to obtain the correct type:

```
return_expr = (return_kw? -> ()) expr
return_kw = "return" spacing
```

It breaks our hopes to have a clean grammar and because it is nested, this is a bit difficult to understand at a first sight. This is why we have introduced a new type `(^)` called the "invisible" type. It is a unit type that can not be composed within another type, it becomes _invisible_ in tuple type for example. The grammar then becomes:

```
return_expr = return_kw? expr
return_kw = "return" spacing -> (^)
```

The type of `return_expr` is the type of `expr` as expected. The circumflex symbol in `(^)` indicates a bottom up propagation of unit in expression. The propagation is only stopped if a value with another type is encountered. The expression `return_kw? return_kw?` has type `(^)`, it has been propagated across `Option<(^)>` and `((^), (^))` types.

That is for the story of typing parsing expressions but don't be sad! It continues in the [issues tracker](https://github.com/ptal/Rust.peg/issues)...

## What's next?

For the moment my priority is to stabilize/test things and to add a decent error reporting mechanism, probably something based on the article [Error reporting in parsing expression grammars](http://arxiv.org/abs/1405.6646). Next I want more static analysis to prevent grammar design error such as in `"=" / "=="` (can you find what's wrong?) Here some other wanted features:

* Automatic wrapping of values into `Spanned<T>` structure to get location information ([#13](https://github.com/ptal/Rust.peg/issues/13)).
* Closest relation between host language types and grammar expression types, for example `e1 > A / e2 > B` with `A` and `B` being variants ([#41](https://github.com/ptal/Rust.peg/issues/41), [#53](https://github.com/ptal/Rust.peg/issues/53), [#54](https://github.com/ptal/Rust.peg/issues/54)).
* Extend the choice operator to handle erroneous cases ([#30](https://github.com/ptal/Rust.peg/issues/30)).
* Bootstrap the grammar ([#42](https://github.com/ptal/Rust.peg/issues/42)).
* Parametrize rules with other rules and arguments ([#10](https://github.com/ptal/Rust.peg/issues/10), [#12](https://github.com/ptal/Rust.peg/issues/12), [#28](https://github.com/ptal/Rust.peg/issues/28)).
* [...](https://github.com/ptal/Rust.peg/issues)

A shortcoming to cleanly achieve these objectives with the Rust compiler is that we can only access item definitions declared inside the procedural macro. It probably means that, for the moment, compositionality would come at the cost of some run-time verifications (or no inter-grammar analysis at all).
