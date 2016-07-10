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
use rust::TokenTree;

pub struct RepeatCompiler
{
  expr_idx: usize,
  cardinality_min: usize,
  compiler_kind: CompilerKind
}

impl RepeatCompiler
{
  pub fn recognizer(expr_idx: usize, cardinality_min: usize) -> RepeatCompiler {
    RepeatCompiler {
      expr_idx: expr_idx,
      cardinality_min: cardinality_min,
      compiler_kind: CompilerKind::Recognizer
    }
  }

  pub fn parser(expr_idx: usize, cardinality_min: usize) -> RepeatCompiler {
    RepeatCompiler {
      expr_idx: expr_idx,
      cardinality_min: cardinality_min,
      compiler_kind: CompilerKind::Parser
    }
  }

  fn compile<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation, exit_label: TokenTree, body: RExpr) -> RExpr
  {
    let mark = context.next_mark_name();

    continuation.map_success(|success, _|
      quote_expr!(context.cx(),
        {
          let mut $mark = state.mark();
          $exit_label: loop {
            $body
            $mark = state.mark();
          }
          state.restore($mark);
          $success
        }
      )
    )
    .unwrap_success()
  }

  fn compile_recognizer<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation, exit_label: TokenTree) -> RExpr
  {
    let body =
      Continuation::new(
        quote_expr!(context.cx(), ()),
        quote_expr!(context.cx(), break $exit_label)
      )
      .compile_success(context, recognizer_compiler, self.expr_idx)
      .unwrap_success();
    self.compile(context, continuation, exit_label, body)
  }

  fn compile_parser<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation, exit_label: TokenTree) -> RExpr
  {
    let result_var = context.next_unbounded_var();
    let vars_names = context.open_scope(self.expr_idx);
    context.push_closure_arg(result_var);
    let span = context.cx().call_site();
    let result_value = tuple_value(context.cx(), span, vars_names);
    let body =
      Continuation::new(
        quote_expr!(context.cx(), $result_var.push($result_value)),
        quote_expr!(context.cx(), break $exit_label)
      )
      .compile_success(context, parser_compiler, self.expr_idx)
      .unwrap_success();
    let repeat_expr = self.compile(context, continuation, exit_label, body);
    context.close_scope();
    quote_expr!(context.cx(),
      {
        let mut $result_var = vec![];
        $repeat_expr
      }
    )
  }
}

impl CompileExpr for RepeatCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let exit_label = context.next_exit_label();
    match self.compiler_kind {
      CompilerKind::Recognizer => self.compile_recognizer(context, continuation, exit_label),
      CompilerKind::Parser => self.compile_parser(context, continuation, exit_label)
    }
  }
}
