// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/*
Compile the typed AST into Rust code.
It generates a recognizer and parser function for each rules.
We compile the PEG expressions by continuation-passing style.
Basically, it means that the code is generated from bottom-up, right to left.
Every PEG expression implements the `CompileExpr` trait which takes a continuation and a context, and returns an Rust expression.
A continuation contains the `success` and `failure` components, which are both Rust expression.
Basically, when compiling an expression, `success` is the expression to call if the parsing succeeds, and `failure` the one to call otherwise.

The compilation is not easy to understand and we only explain it briefly here.
A good strategy is to write a simple rule such as `test = (. / .) "a"` and to check what is the generated code with `cargo expand parse_test`.

We give the general compilation scheme of different operators below.
We write `[e](s,f)` the compilation of the expression `e` with the success continuation `s` and the failed continuation `f`.

```
  [r:(T1,...,TN) = e] = [e](s,f)
    s = state.success((v1,v2,...,vN))
    f = state.failure()

  ["a"](s,f) = if state.consume_prefix("a") { s } else { state.error("a"); f }

  [e1 e2](s,f) = [e1]([e2](s,f), f)

  [e?](s,f) = {
    let mut result = None;
    let mark1 = state.mark();
    state = [e](
      (vI = Some((u1,...,uN)); state),
      f)
    if state.is_failed() { state = state.restore_from_failure(mark1); }
    s
  }

```

A first problem is the name collision.
Indeed, we declare `result` and `mark1` when compiling `e?` but maybe the same names are already used in the code outside of this expression.
We avoid this problem by generating fresh variables, thanks to the structure `name_factory::NameFactory`, which is kept in the `context` structure.

The main challenge comes from the choice expression.
A simple compilation would be as follows:

```
  [e1 / e2](s,f) = {
    let mark1 = state.mark();
    state = [e1](s,f);
    if state.is_failed() {
      state = state.restore_from_failure(mark1);
      state = [e2](s,f);
      if state.is_failed() {
        f
      }
      else { state }
    }
    else { state }
  }
```

The problem with this strategy is that the expression `s` is duplicated for each branch of the choice expression.
Imagine an expression such as `(e1 / e2) (e3 / e4) (e5 / e6) e7`, then you repeat the code for `e7` 8 times (for each branch combination), the code for `(e5 / e6)` 4 times and the one for `(e3 / e4)` 2 times.
Therefore, it is not very ideal.
Note that the function `Context::do_not_duplicate_success` allows this behavior if it returns `true`.

The strategy is to encapsulate the success continuation in a closure:

```
  [e1 / e2](s,f) = {
    let cont = |mut state: ParseState<Stream<'a>, T>, v1: T1, ..., vN: TN| {
      s
    };
    let mark1 = state.mark();
    state = [e1](cont(v1,...,vN),f);
    if state.is_failed() {
      state = state.restore_from_failure(mark1);
      state = [e2](cont(v1,...,vN),f);
      if state.is_failed() {
        f
      }
      else { state }
    }
    else { state }
  }
```

The variables `v1...vN` are those occurring free in the success continuation `s`.
We must keep the names and types of the variables `v1...vN` somewhere, and this is the purpose of `context.free_variables`.

Another subtlety is the usage of context for free variables.
For instance, a value of type `(T1, Option<(T2, T3)>)` is given by two variables, let's say `(x, y)`.
However, to build `y` of type `Option<(T2, T3)>`, we need two more names `w, z`, so we can build `Some((w, z))`.
Thus, the free variables are only interesting for a given scope, and when building the full expression.
Scopes are managed by the `context.open_scope` and `context.close_scope` functions.
We open a new scope when crossing a type such as Option or List, but also for semantic actions.
When building the value `(T2, T3)`, we only care about `w` and `z`, hence if a continuation function should be build for `e`, we only need to pass `w` and `z` to it, and not `x` and `y`.
*/

mod context;
mod continuation;
mod name_factory;
mod compiler;

use middle::typing::ast::*;

pub fn compile(grammar: TGrammar) -> proc_macro2::TokenStream
{
  compiler::GrammarCompiler::compile(grammar)
}
