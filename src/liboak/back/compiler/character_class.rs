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

type VarInPatternFn = for <'a, 'b, 'c> fn(&mut Context<'a, 'b, 'c>) -> Ident;

fn bind_x_var<'a, 'b, 'c>(context: &mut Context<'a, 'b, 'c>) -> Ident {
  context.cx().ident_of("x")
}

fn bind_var<'a, 'b, 'c>(context: &mut Context<'a, 'b, 'c>) -> Ident {
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

  fn compile_interval(&self, cx: &ExtCtxt,
    char_interval: CharacterInterval, x: Ident) -> RExpr
  {
    let CharacterInterval{lo, hi} = char_interval;
    quote_expr!(cx, ($x >= $lo && $x <= $hi))
  }

  fn compile_condition(&self, cx: &ExtCtxt, x: Ident) -> RExpr {
    let mut intervals = self.classes.intervals.iter().cloned();
    let first_interval = intervals.next()
      .expect("Empty character intervals should be forbidden at the parsing stage.");
    intervals
      .map(|char_interval| self.compile_interval(cx, char_interval, x))
      .fold(
        self.compile_interval(cx, first_interval, x),
        |accu, interval| quote_expr!(cx, $accu || $interval)
      )
  }
}

impl CompileExpr for CharacterClassCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let cx = context.cx();

    let classes_desc = format!("{}", self.classes);
    let classes_desc_str = classes_desc.as_str();

    let var = (self.bounded_var)(context);
    let condition = self.compile_condition(cx, var);
    let mark = context.next_mark_name();
    continuation
      .map_success(|success, failure| quote_expr!(cx, {
        let $mark = state.mark();
        match state.next() {
          Some($var) if $condition => {
            $success
          }
          _ => {
            state = state.restore($mark);
            state.error($classes_desc_str);
            $failure
          }
        }
      }))
     .unwrap_success()
  }
}
