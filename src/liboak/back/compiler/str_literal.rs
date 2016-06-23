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
  pub fn new(literal: String) -> StrLiteralCompiler {
    StrLiteralCompiler {
      literal: literal
    }
  }
}

impl CompileParser for StrLiteralCompiler
{
  fn compile_parser<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr {
    self.compile_recognizer(context)
  }

  fn compile_recognizer<'a, 'b, 'c>(&self, context: Context<'a, 'b, 'c>) -> RExpr {
    let lit = self.literal.as_str();
    let success = context.success;
    let failure = context.failure;
    quote_expr!(context.grammar.cx,
      if state.consume_prefix($lit) {
        $success
      }
      else {
        state.error($lit);
        $failure
      })
  }
}
