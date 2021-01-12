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

pub struct SpannedExprCompiler{
  expr_idx: usize,
  is_ranged: bool
}

impl SpannedExprCompiler
{
  pub fn parser(expr_idx: usize, is_ranged: bool) -> SpannedExprCompiler {
    SpannedExprCompiler { expr_idx, is_ranged }
  }
}

impl CompileExpr for SpannedExprCompiler
{
  fn compile_expr<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let lo_sp = context.next_mark_name();
    // The `n` next variable belongs to expr_idx so we pop the next one after these.
    let result = context.next_free_var_skip(self.expr_idx);

    let mut result_expr: syn::Expr = parse_quote!(
      Range { start: #lo_sp.clone(), end: state.mark() }
    );
    if !self.is_ranged {
      result_expr = parse_quote!((#result_expr).stream_span());
    }

    context.push_mark(lo_sp.clone());

    let spanned_expr = continuation
      .map_success(|success, _| {
        parse_quote!({
          let #result = #result_expr;
          #success
        })
      })
      .compile_success(context, parser_compiler, self.expr_idx)
      .unwrap_success();
    context.pop_mark();
    parse_quote!({
      let #lo_sp = state.mark();
      #spanned_expr
    })
  }
}
