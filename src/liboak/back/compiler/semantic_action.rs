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

pub struct SemanticActionCompiler
{
  expr_idx: usize,
  boxed: bool,
  action: syn::Expr
}

impl SemanticActionCompiler
{
  pub fn parser(expr_idx: usize, boxed: bool, action: syn::Expr, _this_idx: usize) -> SemanticActionCompiler {
    SemanticActionCompiler {
      expr_idx, boxed, action
    }
  }
}

impl CompileExpr for SemanticActionCompiler
{
  fn compile_expr<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let result = context.next_free_var();
    let scope = context.open_scope(self.expr_idx);
    let args: Vec<syn::Expr> = context.free_variables().into_iter()
      .map(|var| parse_quote!(#var))
      .collect();
    let action = self.action.clone();
    let is_unit_variant =
      match self.action {
        syn::Expr::Path(ref expr_path) =>
          if let Some(x) = expr_path.path.segments.last() {
            x.ident.to_string().chars().next().expect("non empty identifier").is_uppercase() &&
            context.has_unit_type(self.expr_idx)
          }
          else { false }
        _ => false
      };
    let expr = continuation
      .map_success(|success, _| {
        let action_call: syn::Expr =
          if is_unit_variant {
            parse_quote!(#action)
          }
          else {
            parse_quote!(#action(#(#args),*))
          };
        let boxed_action_call: syn::Expr =
          if self.boxed { parse_quote!(Box::new(#action_call)) }
          else { action_call };
        parse_quote!({
          let #result = #boxed_action_call;
          #success
        })})
      .compile_success(context, parser_compiler, self.expr_idx)
      .unwrap_success();
    context.close_scope(scope);
    expr
  }
}
