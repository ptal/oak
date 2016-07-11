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

pub struct OptionalCompiler
{
  expr_idx: usize,
  compiler_kind: CompilerKind
}

impl OptionalCompiler
{
  pub fn recognizer(expr_idx: usize) -> OptionalCompiler {
    OptionalCompiler {
      expr_idx: expr_idx,
      compiler_kind: CompilerKind::Recognizer
    }
  }

  pub fn parser(expr_idx: usize) -> OptionalCompiler {
    OptionalCompiler {
      expr_idx: expr_idx,
      compiler_kind: CompilerKind::Parser
    }
  }

  fn compile<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation, body: RExpr) -> RExpr
  {
    let mark = context.next_mark_name();
    continuation
      .map_success(|success, _|
        quote_expr!(context.cx(), {
          let $mark = state.mark();
          state = $body;
          if state.is_failed() {
            state = state.restore_from_failure($mark);
          }
          $success
        })
      )
      .unwrap_success()
  }

  fn compile_recognizer<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let body = context.compile_recognizer_expr(self.expr_idx);
    self.compile(context, continuation, body)
  }

  fn value_constructor(cx: &ExtCtxt, result_var: Ident, result_value: RExpr) -> RExpr {
    quote_expr!(cx, {
      $result_var = Some($result_value);
      state
    })
  }

  fn compile_parser<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let ty = quote_ty!(context.cx(), Option<_>);
    let (body, result_var) = context.value_constructor(
      self.expr_idx,
      ty,
      OptionalCompiler::value_constructor
    );
    let optional_expr = self.compile(context, continuation, body);
    quote_expr!(context.cx(), {
      let mut $result_var = None;
      $optional_expr
    })
  }
}

impl CompileExpr for OptionalCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    match self.compiler_kind {
      CompilerKind::Recognizer => self.compile_recognizer(context, continuation),
      CompilerKind::Parser => self.compile_parser(context, continuation)
    }
  }
}
