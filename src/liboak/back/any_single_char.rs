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

pub struct AnySingleCharCompiler
{
  matched_pattern: for <'a, 'b, 'c> fn(&mut Context<'a, 'b, 'c>) -> RPat
}

impl AnySingleCharCompiler
{
  pub fn recognizer() -> AnySingleCharCompiler {
    AnySingleCharCompiler {
      matched_pattern: AnySingleCharCompiler::ignore_value
    }
  }

  pub fn parser() -> AnySingleCharCompiler {
    AnySingleCharCompiler {
      matched_pattern: AnySingleCharCompiler::bind_value
    }
  }

  #[allow(unused_imports)]
  fn ignore_value<'a, 'b, 'c>(context: &mut Context<'a, 'b, 'c>) -> RPat {
    quote_pat!(context.cx(), _)
  }

  fn bind_value<'a, 'b, 'c>(context: &mut Context<'a, 'b, 'c>) -> RPat {
    let value_name = context.next_data_name();
    quote_pat!(context.cx(), $value_name)
  }
}

impl CompileExpr for AnySingleCharCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let pattern = (self.matched_pattern)(context);
    continuation
      .map_success(|success, failure| quote_expr!(context.cx(),
        match state.next() {
          Some($pattern) => {
            $success
          }
          None => {
            state.error("<character>");
            $failure
          }
        }
      ))
     .unwrap_success()
  }
}
