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

  fn compile<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation, body: syn::Expr) -> syn::Expr
  {
    let mark = context.next_mark_name();
    continuation.map_success(|success, failure|
      if self.cardinality_min > 0 {
        let counter = context.next_counter_name();
        let cardinality_min = self.cardinality_min;
        parse_quote!(
          {
            let mut #mark = state.mark();
            let mut #counter = 0;
            loop {
              state = #body;
              if state.is_successful() {
                #counter += 1;
                #mark = state.mark();
              }
              else {
                break;
              }
            }
            if #counter < #cardinality_min {
              #failure
            }
            else {
              let mut state = state.restore_from_failure(#mark);
              #success
            }
          }
        )
      }
      else {
        parse_quote!(
          {
            let mut #mark = state.mark();
            loop {
              state = #body;
              if state.is_successful() {
                #mark = state.mark();
              }
              else {
                break;
              }
            }
            let mut state = state.restore_from_failure(#mark);
            #success
          }
        )
      }
    )
    .unwrap_success()
  }

  fn compile_recognizer<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let body = context.compile_recognizer_expr(self.expr_idx);
    self.compile(context, continuation, body)
  }

  fn value_constructor(result_var: Ident, result_value: syn::Expr) -> syn::Expr {
    parse_quote!({
      #result_var.push(#result_value);
      state
    })
  }

  fn compile_parser<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let ty: syn::Type = parse_quote!(Vec<_>);
    let (body, result_var) = context.value_constructor(
      self.expr_idx,
      ty,
      RepeatCompiler::value_constructor
    );
    let repeat_expr = self.compile(context, continuation, body);
    parse_quote!({
      let mut #result_var = vec![];
      #repeat_expr
    })
  }
}

impl CompileExpr for RepeatCompiler
{
  fn compile_expr<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    match self.compiler_kind {
      CompilerKind::Recognizer => self.compile_recognizer(context, continuation),
      CompilerKind::Parser => self.compile_parser(context, continuation)
    }
  }
}
