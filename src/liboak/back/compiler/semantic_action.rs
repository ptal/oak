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
use rust::AstBuilder;

pub struct SemanticActionCompiler
{
  expr_idx: usize,
  action: Ident,
  this_idx: usize
}

impl SemanticActionCompiler
{
  pub fn parser(expr_idx: usize, action: Ident, this_idx: usize) -> SemanticActionCompiler {
    SemanticActionCompiler {
      expr_idx: expr_idx,
      action: action,
      this_idx: this_idx
    }
  }
}

impl CompileExpr for SemanticActionCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let cx = context.cx();
    let result = context.next_free_var();
    let scope = context.open_scope(self.expr_idx);
    let span = context.expr_span(self.this_idx);
    let args: Vec<_> = context.free_variables().into_iter()
      .map(|var| quote_expr!(cx, $var))
      .collect();
    let action_call = cx.expr_call_ident(span, self.action, args);
    let expr = continuation
      .map_success(|success, _|
        quote_expr!(cx, {
          let $result = $action_call;
          $success
        })
      )
      .compile_success(context, parser_compiler, self.expr_idx)
      .unwrap_success();
    context.close_scope(scope);
    expr
  }
}
