% The Story of Oak

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
