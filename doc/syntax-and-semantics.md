% Syntax and Semantics

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
| `e1 e2 e3`      | `(T1, T2, T3)`        | 3                | Match `e1 e2 e3` in sequence. Immediately fails when one fails. |
| `e > f`         | Return type of `f`    | 4                | Match `e` and if it succeeds, call `f(v)` where `v` is the value of `e`. |
| `e -> ()`       | `()`                  | 4                | Force the type of `e` to be `()`. |
| `e -> (^)`      | `(^)`                 | 4                | Force the type of `e` to be `(^)`. |
| `e1 / e2 / e3`  | Type of any `e`       | 5                | Match `e1 e2 e3` in sequence. Immediately succeeds when one succeeds. |

## Introduction to expressions types and `(^)`

The full explanation of the what and why of types is available in the section [The Story of Oak](the-story-of-oak.md). A goal of this library is to give a type to any expression grammar. This permits to call a semantic action without naming expressions. In some cases, for example with the spacing rule `spacing = [" \n\t"]*`, the grammar compiler will not generate the expected type, here the rule `spacing` has type `Vec<char>` instead of `()` — we usually do not care about spaces. Therefore, users must annotate expressions with `e -> ()` to force their types to be `()`. It works and is enough for most cases. However, we sometimes want to propagate unit type up in the expression tree because these expressions are only of syntactic interest. The fact is that `e?` has type `Option<T>` even if `T = ()`. It is expected since `Option<()>` carries a boolean information about the presence of something. If we do not care, we can annotate `e` with `(^)` and the unit type will automatically be propagated, and even `e1? e2*` will have type `(^)` if `e1` and `e2` have type `(^)`. In the end, the goal is really to give an expression the type that you expect it to have!