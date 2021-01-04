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

pub enum Kind {
  Not, And
}

pub struct SyntacticPredicateCompiler
{
  expr_idx: usize,
  kind: Kind
}

impl SyntacticPredicateCompiler
{
  pub fn recognizer(expr_idx: usize, kind: Kind) -> SyntacticPredicateCompiler {
    SyntacticPredicateCompiler {
      expr_idx: expr_idx,
      kind: kind
    }
  }

  pub fn compile<'a>(&self, context: &mut Context<'a>,
    success_case: syn::Expr, failure_case: syn::Expr) -> syn::Expr
  {
    let mark = context.next_mark_name();
    let expr = Continuation::new(
        parse_quote!(state),
        parse_quote!(state.failure())
      )
      .compile_success(context, recognizer_compiler, self.expr_idx)
      .unwrap_success();
    parse_quote!(
      {
        let #mark = state.mark();
        state = #expr;
        let is_success = state.is_successful();
        state = state.restore(#mark);
        if is_success {
          #success_case
        }
        else {
          #failure_case
        }
      }
    )
  }
}

impl CompileExpr for SyntacticPredicateCompiler
{
  fn compile_expr<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let (success, failure) = continuation.unwrap();
    match self.kind {
      Kind::Not => self.compile(context, failure, success),
      Kind::And => self.compile(context, success, failure)
    }
  }
}
