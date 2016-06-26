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
use rust;

pub struct AnySingleCharCompiler
{
  matched_value: fn(&ExtCtxt, &mut NameFactory) -> Vec<rust::TokenTree>
}

impl AnySingleCharCompiler
{
  pub fn recognizer() -> AnySingleCharCompiler {
    AnySingleCharCompiler {
      matched_value: ignore_value
    }
  }

  pub fn parser() -> AnySingleCharCompiler {
    AnySingleCharCompiler {
      matched_value: bind_value
    }
  }
}

impl CompileExpr for AnySingleCharCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr {
    let success = context.success;
    let failure = context.failure;
    let value = (self.matched_value)(context.grammar.cx, context.name_factory);
    quote_expr!(context.grammar.cx,
      match state.next() {
        Some($value) => {
          $success
        }
        None => {
          state.error("<character>");
          $failure
        }
      }
    )
  }
}

#[allow(unused_imports)]
fn ignore_value(cx: &ExtCtxt, _: &mut NameFactory) -> Vec<rust::TokenTree> {
  quote_tokens!(cx, _)
}

fn bind_value(cx: &ExtCtxt, name_factory: &mut NameFactory) -> Vec<rust::TokenTree> {
  let value_name = name_factory.next_data_name();
  quote_tokens!(cx, $value_name)
}
