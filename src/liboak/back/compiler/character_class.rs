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
use quote::format_ident;

type VarInPatternFn = for <'a> fn(&mut Context<'a>) -> Ident;

fn bind_x_var<'a>(_context: &mut Context<'a>) -> Ident {
  format_ident!("x")
}

fn bind_var<'a>(context: &mut Context<'a>) -> Ident {
  context.next_free_var()
}

pub struct CharacterClassCompiler
{
  classes: CharacterClassExpr,
  bounded_var: VarInPatternFn
}

impl CharacterClassCompiler
{
  pub fn recognizer(classes: CharacterClassExpr) -> CharacterClassCompiler {
    CharacterClassCompiler {
      classes: classes,
      bounded_var: bind_x_var
    }
  }

  pub fn parser(classes: CharacterClassExpr) -> CharacterClassCompiler {
    CharacterClassCompiler {
      classes: classes,
      bounded_var: bind_var
    }
  }

  fn compile_interval(&self, char_interval: CharacterInterval, x: Ident) -> syn::Expr
  {
    let CharacterInterval{lo, hi} = char_interval;
    parse_quote!((#x >= #lo && #x <= #hi))
  }

  fn compile_condition(&self, x: Ident) -> syn::Expr {
    let mut intervals = self.classes.intervals.iter().cloned();
    let first_interval = intervals.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    intervals
      .map(|char_interval| self.compile_interval(char_interval, x.clone()))
      .fold(
        self.compile_interval(first_interval, x.clone()),
        |accu, interval| parse_quote!(#accu || #interval)
      )
  }
}

impl CompileExpr for CharacterClassCompiler
{
  fn compile_expr<'a>(&self, context: &mut Context<'a>,
    continuation: Continuation) -> syn::Expr
  {
    let classes_desc = format!("{}", self.classes);
    let classes_desc_str = classes_desc.as_str();

    let var = (self.bounded_var)(context);
    let condition = self.compile_condition(var.clone());
    let mark = context.next_mark_name();
    continuation
      .map_success(|success, failure| parse_quote!({
        let #mark = state.mark();
        match state.next() {
          Some(#var) if #condition => {
            #success
          }
          _ => {
            state = state.restore(#mark);
            state.error(#classes_desc_str);
            #failure
          }
        }
      }))
     .unwrap_success()
  }
}
