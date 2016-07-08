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

pub struct StrLiteralCompiler
{
  literal: String
}

impl StrLiteralCompiler
{
  pub fn recognizer(literal: String) -> StrLiteralCompiler {
    StrLiteralCompiler {
      literal: literal
    }
  }

  pub fn parser(literal: String) -> StrLiteralCompiler {
    StrLiteralCompiler::recognizer(literal)
  }
}

impl CompileExpr for StrLiteralCompiler
{
  fn compile_expr<'a, 'b, 'c>(&self, context: &mut Context<'a, 'b, 'c>,
    continuation: Continuation) -> RExpr
  {
    let lit = self.literal.as_str();
    continuation
      .map_success(|success, failure| quote_expr!(context.cx(),
        if state.consume_prefix($lit) {
          $success
        }
        else {
          state.error($lit);
          $failure
        }
      ))
      .unwrap_success()
  }
}
