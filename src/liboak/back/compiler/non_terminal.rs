// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use back::compiler::*;
use back::compiler::value::*;
use back::name_factory::*;

pub struct NonTerminalCompiler;

impl NonTerminalCompiler
{
  pub fn recognizer(id: Ident) -> NonTerminalRecognizerCompiler {
    NonTerminalRecognizerCompiler {
      id: id
    }
  }

  pub fn parser(id: Ident, this_idx: usize) -> NonTerminalParserCompiler {
    NonTerminalParserCompiler {
      id: id,
      this_idx: this_idx
    }
  }
}

pub struct NonTerminalRecognizerCompiler
{
  id: Ident
}

impl CompileExpr for NonTerminalRecognizerCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let recognizer_fn = recognizer_name(context.cx(), self.id);
    continuation
      .map_success(|success, failure| quote_expr!(context.cx(),
        {
          state = $recognizer_fn(state);
          if state.is_successful() {
            state.discard_data();
            $success
          }
          else {
            $failure
          }
        }
      ))
      .unwrap_success()
  }
}

pub struct NonTerminalParserCompiler
{
  id: Ident,
  this_idx: usize
}

impl CompileExpr for NonTerminalParserCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let cx = context.cx();
    let parser_fn = parser_name(cx, self.id);
    let cardinality = context.expr_cardinality(self.this_idx);
    let mut vars_names: Vec<_> = (0..cardinality)
      .map(|_| context.next_free_var())
      .collect();
    // Due to the reverse compilation scheme, variables are given as `a3, a2,...`, however we need to match them in the good order.
    // Note that we cannot use `rev()` since we depend on a global state.
    vars_names.reverse();
    let vars = tuple_pattern(cx, context.expr_span(self.this_idx), vars_names);
    continuation
      .map_success(|success, failure| quote_expr!(cx,
        {
          let stateful = $parser_fn(state);
          if stateful.is_successful() {
            let (stateless, $vars) = stateful.extract_data();
            state = stateless;
            $success
          }
          else {
            state = stateful.failure();
            $failure
          }
        }
      ))
      .unwrap_success()
  }
}
